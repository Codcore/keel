#!/usr/bin/env python3
"""The tool speaks the project's language, and says the same thing in both."""

import ast
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import keel  # noqa: E402


def spoken_strings():
    """Every literal handed to t() in the tool, read out of the source itself.

    Both guards below are defined against this one set: widen the extraction —
    to catch t() reached through an alias, say — and the pair widens together.
    Two copies would let one of them keep checking the narrower set in silence.
    """
    source = os.path.join(os.path.dirname(os.path.dirname(
        os.path.abspath(__file__))), "keel.py")
    with open(source, encoding="utf-8") as handle:
        tree = ast.parse(handle.read())
    constants = {node.targets[0].id: node.value.value
                 for node in tree.body
                 if isinstance(node, ast.Assign) and len(node.targets) == 1
                 and isinstance(node.targets[0], ast.Name)
                 and isinstance(node.value, ast.Constant)
                 and isinstance(node.value.value, str)}
    spoken = set()
    for node in ast.walk(tree):
        if not (isinstance(node, ast.Call) and isinstance(node.func, ast.Name)
                and node.func.id == "t" and node.args):
            continue
        head = node.args[0]
        if isinstance(head, ast.Constant) and isinstance(head.value, str):
            spoken.add(head.value)
        elif isinstance(head, ast.Name) and head.id in constants:
            # t(WHY_HINT): a named constant is spoken too — without this, a
            # string that moves into a constant would read as a dead entry.
            spoken.add(constants[head.id])
    return spoken


class TestCatalogue(unittest.TestCase):
    """The catalogue is keyed by English, so a gap degrades to readable English."""

    def setUp(self):
        self.addCleanup(setattr, keel, "OUTPUT_LANG", keel.OUTPUT_LANG)

    def test_english_is_the_default(self):
        self.assertEqual(keel.DEFAULTS["lang"], "en")
        self.assertEqual(keel.t("clean"), "clean")

    def test_ukrainian_comes_from_the_catalogue(self):
        keel.OUTPUT_LANG = "uk"
        self.assertEqual(keel.t("clean"), "чисто")

    def test_a_missing_translation_falls_back_to_english(self):
        keel.OUTPUT_LANG = "uk"
        self.assertEqual(keel.t("never translated"), "never translated")

    def test_fields_are_filled_in_both(self):
        for lang, expected in (("en", "lib/x.ex"), ("uk", "lib/x.ex")):
            keel.OUTPUT_LANG = lang
            self.assertIn(expected, keel.t("changed but not declared: {name}",
                                           name="lib/x.ex"))

    def test_every_translation_keeps_the_same_placeholders(self):
        """Загублене поле в перекладі — це KeyError на живому проєкті."""
        for english, ukrainian in keel.UK.items():
            self.assertEqual(set(re.findall(r"\{(\w+)\}", english)),
                             set(re.findall(r"\{(\w+)\}", ukrainian)),
                             f"поля розійшлись: {english[:50]}")

    def test_every_check_name_is_translated(self):
        for name in keel.CHECK_NAMES.values():
            self.assertIn(name, keel.UK, f"немає перекладу: {name}")

    def test_every_string_that_reaches_a_person_is_in_the_catalogue(self):
        """Пропущений запис не падає — він тихо виходить англійською.

        Це той самий тихий клас, що й зелена перевірка над неперевіреним: у
        проєкті з lang: uk половина фрази українською, половина ні, і ніщо
        про це не каже.
        """
        missing = sorted(text for text in spoken_strings() if text not in keel.UK)
        self.assertEqual(missing, [], f"без перекладу: {len(missing)}")

    def test_the_catalogue_holds_nothing_the_tool_never_says(self):
        """Запис без виклику — слід перейменованої фрази, і він старіє мовчки."""
        said = spoken_strings() | set(keel.CHECK_NAMES.values())
        stale = sorted(text for text in keel.UK if text not in said)
        self.assertEqual(stale, [], f"мертвих записів: {len(stale)}")

    def test_the_catalogue_literal_has_no_duplicate_keys(self):
        """Пізніший дубль тихо затирає ранішній — словник цього не покаже."""
        source = os.path.join(os.path.dirname(os.path.dirname(
            os.path.abspath(__file__))), "keel.py")
        with open(source, encoding="utf-8") as handle:
            tree = ast.parse(handle.read())
        for node in ast.walk(tree):
            if (isinstance(node, ast.Assign)
                    and getattr(node.targets[0], "id", "") == "UK"):
                seen = {}
                for key in node.value.keys:
                    text = key.value
                    self.assertNotIn(text, seen,
                                     f"дубль ключа (рядки {seen.get(text)} "
                                     f"і {key.lineno}): {text[:60]!r}")
                    seen[text] = key.lineno

    def test_no_translation_is_left_identical(self):
        """Однаковий рядок означає забутий переклад, а не збіг."""
        same = [en for en, uk in keel.UK.items() if en == uk]
        self.assertEqual(same, [])


class TestWhatIsWrittenIntoAProject(unittest.TestCase):
    """Generated files travel to strangers; none of them may be a stray language."""

    def test_the_ci_workflow_is_english(self):
        text = keel.CI_TEMPLATE.format(tool=keel.VENDORED, setup="")
        self.assertNotRegex(text, "[\u0400-\u04FF]")

    def test_the_git_hook_script_is_english(self):
        for name in keel.HOOKS:
            script = keel.hook_script(name, "/somewhere/keel.py")
            self.assertNotRegex(script, "[\u0400-\u04FF]", name)

    def test_the_failure_message_a_stranger_meets_is_english(self):
        """Його бачать на зламаному пуші — найгірший момент для чужої мови."""
        script = keel.hook_script("pre-push", "/somewhere/keel.py")
        self.assertIn("no tool found", script)

    def test_the_skill_files_are_english(self):
        for skill in keel.SKILLS:
            for agent, relative in keel.skill_targets(skill):
                for lang in keel.LANGS:
                    body = keel.render_skill(skill, agent, lang).split("---", 2)[2]
                    self.assertNotRegex(body, "[\u0400-\u04FF]", relative)


class TestTheTemplatesFollowTheLanguage(unittest.TestCase):
    """`keel new` writes into somebody's project; it may not write in a stray one."""

    def setUp(self):
        self.addCleanup(setattr, keel, "OUTPUT_LANG", keel.OUTPUT_LANG)

    def render(self, lang):
        keel.OUTPUT_LANG = lang
        return keel.wave_skeleton("0001-thing"), keel.contract_skeleton()

    def test_english_by_default(self):
        wave, contract = self.render("en")
        self.assertIn("## Why", wave)
        self.assertIn("Boundaries:", wave)
        self.assertNotRegex(wave, "[\u0400-\u04FF]")
        self.assertNotRegex(contract, "[\u0400-\u04FF]")

    def test_ukrainian_when_asked(self):
        wave, contract = self.render("uk")
        self.assertIn("## Навіщо", wave)
        self.assertIn("Межі:", wave)
        self.assertIn("Модуль", contract)

    def test_the_header_shape_is_the_same_in_both(self):
        """Поля стають кодом — мова їх не чіпає."""
        for lang in keel.LANGS:
            wave, contract = self.render(lang)
            for field in ("depends_on:", "scenarios:", "transforms:",
                          "implements:", "contracts:", "files:"):
                self.assertIn(field, wave, f"{lang}: {field}")
            for field in ("module:", "exports:", "verify:"):
                self.assertIn(field, contract, f"{lang}: {field}")

    def test_both_skeletons_parse(self):
        """Шаблон, що не читається власним читачем, — зламаний старт."""
        for lang in keel.LANGS:
            wave, contract = self.render(lang)
            for text in (wave, contract):
                front, _, _ = keel.split_front_matter(text)
                self.assertIsNotNone(front, lang)
                keel.parse_yaml(front)

    def wave(self, text):
        root = tempfile.mkdtemp(prefix="keel-skel-")
        self.addCleanup(shutil.rmtree, root, True)
        folder = os.path.join(root, "keel", "waves")
        os.makedirs(folder)
        path = os.path.join(folder, "0001-thing.md")
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
        return keel.Wave(path, "keel/waves/0001-thing.md")

    def test_the_why_heading_is_read_in_either_language(self):
        """Проєкт може змінити мову — наявні кроки лишаються читаними."""
        for lang in keel.LANGS:
            text, _ = self.render(lang)
            keel.OUTPUT_LANG = "en"
            self.assertTrue(self.wave(text).why.strip(), lang)

    def test_the_untouched_placeholder_is_caught_in_either_language(self):
        """gaps має бачити незаповнене «навіщо», хоч якою мовою його написано."""
        for lang in keel.LANGS:
            text, _ = self.render(lang)
            self.assertTrue(keel.unfilled_why(self.wave(text)), lang)

    def test_a_tool_created_wave_with_an_untouched_why_is_flagged(self):
        """Каркас пише слаг без номера, файл зветься з номером — розпізнавач
        не спрацьовував на жодному створеному інструментом кроці."""
        text, _ = self.render("en")
        self.assertTrue(keel.unfilled_why(self.wave(
            keel.wave_skeleton("0001-thing"))))

    def test_a_filled_in_why_is_not_mistaken_for_the_placeholder(self):
        text, _ = self.render("en")
        filled = text.replace("why this wave exists and what is missing without it",
                              "the loop cannot end without it")
        self.assertFalse(keel.unfilled_why(self.wave(filled)))


class TestSpokenLanguage(unittest.TestCase):
    """A real run, both ways, in a real project."""

    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-lang-")
        self.addCleanup(shutil.rmtree, self.root, True)
        os.makedirs(os.path.join(self.root, "keel", "waves"))
        os.makedirs(os.path.join(self.root, "keel", "contracts"))
        subprocess.run(["git", "init", "-b", "main", "-q", self.root], check=True)
        subprocess.run(["git", "-C", self.root, "-c", "user.email=t@e.com",
                        "-c", "user.name=t", "commit", "-q", "--allow-empty",
                        "-m", "base"], check=True)

    def speak(self, lang, *args):
        with open(os.path.join(self.root, keel.CONFIG_FILE), "w") as handle:
            json.dump({"docs": "en", "lang": lang}, handle)
        done = subprocess.run(
            [sys.executable, os.path.join(keel.home(), "keel.py"),
             "-C", self.root, *args], capture_output=True, text=True)
        return done.stdout + done.stderr

    def test_check_speaks_english_by_default(self):
        english = self.speak("en", "check", "--fast")
        self.assertIn("references lead somewhere", english)
        self.assertIn("clean", english)

    def test_check_speaks_ukrainian_when_asked(self):
        ukrainian = self.speak("uk", "check", "--fast")
        self.assertIn("посилання ведуть кудись", ukrainian)
        self.assertIn("чисто", ukrainian)

    def test_the_two_runs_report_the_same_state(self):
        """Мова міняє слова, не вердикт."""
        english = self.speak("en", "check", "--fast")
        ukrainian = self.speak("uk", "check", "--fast")
        self.assertEqual(english.count("✓"), ukrainian.count("✓"))
        self.assertEqual(english.count("✗"), ukrainian.count("✗"))

    def test_a_broken_document_speaks_it_too(self):
        """Помилка складається при читанні файла — тобто до того, як мова відома."""
        with open(os.path.join(self.root, "keel", "waves", "0001-a.md"), "w",
                  encoding="utf-8") as handle:
            handle.write("---\na: b: c\n---\n\nтекст\n")
        self.assertIn("header does not parse", self.speak("en", "check", "--fast"))
        ukrainian = self.speak("uk", "check", "--fast")
        self.assertIn("шапка не читається", ukrainian)
        self.assertNotIn("header does not parse", ukrainian)

    def test_errors_speak_it_too(self):
        self.assertIn("no such wave", self.speak("en", "show", "0009-nope"))
        self.assertIn("хвилі немає", self.speak("uk", "show", "0009-nope"))

    def test_init_speaks_the_language_it_establishes(self):
        """Команда, що вписує lang: uk, сама говорила англійською."""
        import tempfile as tf, shutil as sh
        root = tf.mkdtemp(prefix="keel-initlang-")
        self.addCleanup(sh.rmtree, root, True)
        subprocess.run(["git", "init", "-b", "main", "-q", root], check=True)
        done = subprocess.run(
            [sys.executable, os.path.join(keel.home(), "keel.py"),
             "-C", root, "init", "--lang", "uk", "--no-commit"],
            capture_output=True, text=True)
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertNotIn("uncommitted changes", done.stdout)
        self.assertIn("незакоміченими змінами", done.stdout)

    def test_the_argument_hint_follows_the_language(self):
        """Підказка аргументу — для оператора, як і тригери."""
        for lang, expected in (("en", "[slug of the new wave]"),
                               ("uk", "[слаг нової хвилі]")):
            text = keel.render_skill(keel.SKILLS[0], "claude", lang)
            self.assertIn(expected, text, lang)

    def test_the_docs_do_not_contradict_the_cli_about_the_metavar(self):
        """README казав `-C ТЕКА`, а CLI друкує `-C DIR` — і тест, і доки тепер
        про одне."""
        for name in ("README.md", os.path.join("docs", "uk", "README.md")):
            with open(os.path.join(keel.home(), name), encoding="utf-8") as handle:
                self.assertNotIn("ТЕКА", handle.read(), name)

    def test_the_command_line_itself_stays_english(self):
        """Прапорці й довідка — словник інтерфейсу, як і самі їхні назви."""
        for lang in ("en", "uk"):
            help_text = self.speak(lang, "--help")
            self.assertIn("work in this directory", help_text)
            self.assertIn("-C DIR", help_text)
            self.assertNotIn("ТЕКА", help_text)


if __name__ == "__main__":
    unittest.main()


class TestTheRenameReachedTheTranslationToo(unittest.TestCase):
    """A step became a wave, and the Ukrainian table kept the retired word.

    Four strings said «крок» where the English said wave — among them the header
    `keel next` prints, so the operator was told the old name for the central
    idea on every single run. The same rename went the other way in the CI
    template and broke it; a rename that lands unevenly is the failure, not the
    direction it misses in.
    """

    RETIRED = re.compile(r"\b[Кк]ро(к|ці|ку|ком|ки|ків)\b")

    def pairs_about_waves(self):
        """English that says the word, not English that interpolates the name:
        `{waves}: the plan is not written yet` carries the names in the
        placeholder, and a translation repeating the noun would be worse."""
        return [(en, uk) for en, uk in keel.UK.items()
                if re.search(r"\bwaves?\b", re.sub(r"\{[^}]*\}", "", en).lower())]

    def test_there_are_such_strings_to_check(self):
        """Otherwise the one below passes by having nothing to look at."""
        self.assertGreater(len(self.pairs_about_waves()), 3)

    def test_each_of_them_names_a_wave(self):
        for en, uk in self.pairs_about_waves():
            self.assertIn("хвил", uk.lower(), en)

    def test_the_retired_word_is_gone_from_the_whole_table(self):
        """Not only from the strings about waves: it is the old name for the
        central idea, and anywhere it survives it teaches the wrong one."""
        for en, uk in keel.UK.items():
            self.assertIsNone(self.RETIRED.search(uk), en)
