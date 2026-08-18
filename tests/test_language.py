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
    return {node.args[0].value for node in ast.walk(tree)
            if isinstance(node, ast.Call)
            and isinstance(node.func, ast.Name) and node.func.id == "t"
            and node.args and isinstance(node.args[0], ast.Constant)
            and isinstance(node.args[0].value, str)}


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

    def test_no_translation_is_left_identical(self):
        """Однаковий рядок означає забутий переклад, а не збіг."""
        same = [en for en, uk in keel.UK.items() if en == uk]
        self.assertEqual(same, [])


class TestSpokenLanguage(unittest.TestCase):
    """A real run, both ways, in a real project."""

    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-lang-")
        self.addCleanup(shutil.rmtree, self.root, True)
        os.makedirs(os.path.join(self.root, "keel", "steps"))
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

    def test_errors_speak_it_too(self):
        self.assertIn("no such step", self.speak("en", "show", "0009-nope"))
        self.assertIn("кроку немає", self.speak("uk", "show", "0009-nope"))

    def test_the_command_line_itself_stays_english(self):
        """Прапорці й довідка — словник інтерфейсу, як і самі їхні назви."""
        for lang in ("en", "uk"):
            help_text = self.speak(lang, "--help")
            self.assertIn("work in this directory", help_text)
            self.assertIn("-C DIR", help_text)
            self.assertNotIn("ТЕКА", help_text)


if __name__ == "__main__":
    unittest.main()
