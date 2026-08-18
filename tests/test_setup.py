#!/usr/bin/env python3
"""init, the two language settings, update, translations."""

import os
import json
import shutil
import subprocess
import tempfile
import unittest.mock
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import keel  # noqa: E402
from tests.support import Args  # noqa: E402




# ─────────────────────────────────────────────────────────────────────────────
# keel init
# ─────────────────────────────────────────────────────────────────────────────

class TestInit(unittest.TestCase):
    """An empty mix project where Keel has not been installed yet."""

    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-init-")
        self.addCleanup(shutil.rmtree, self.root, True)
        with open(os.path.join(self.root, "mix.exs"), "w", encoding="utf-8") as handle:
            handle.write("defmodule Demo.MixProject do\nend\n")
        subprocess.run(["git", "init", "-b", "main", "-q", self.root], check=True)

    def read(self, name):
        with open(os.path.join(self.root, name), encoding="utf-8") as handle:
            return handle.read()

    def write(self, name, text):
        with open(os.path.join(self.root, name), "w", encoding="utf-8") as handle:
            handle.write(text)

    def init(self, **kwargs):
        from io import StringIO
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            code = keel.cmd_init(keel.Project(self.root), Args(
                **{"install": True, "force": False, **kwargs}))
        finally:
            sys.stdout = saved
        return code, stream.getvalue()

    def porcelain(self):
        return subprocess.run(
            ["git", "-C", self.root, "status", "--porcelain", "--untracked-files=all"],
            capture_output=True, text=True).stdout

    def log(self):
        return subprocess.run(["git", "-C", self.root, "log", "--format=%s"],
                              capture_output=True, text=True).stdout

    def test_init_commits_only_its_own_files(self):
        """Чуже незакомічене поруч лишається чужим."""
        self.write("mine.txt", "моє, ще не закомічене\n")
        self.init()
        done = subprocess.run(["git", "-C", self.root, "status", "--porcelain",
                               "--untracked-files=all"], capture_output=True, text=True)
        self.assertIn("mine.txt", done.stdout)
        self.assertNotIn("AGENTS.md", done.stdout)

    def test_init_refuses_without_git(self):
        root = tempfile.mkdtemp(prefix="keel-nogit-")
        self.addCleanup(shutil.rmtree, root, True)
        with self.assertRaises(SystemExit):
            keel.cmd_init(keel.Project(root), Args(
                install=True, force=False, docs=None, lang=None, no_commit=False))

    def test_init_says_where_the_agent_must_be_started(self):
        """Скіли беруться з теки старту; запуск вище — і їх немає."""
        _, out = self.init()
        self.assertIn("у самій теці проєкту", out)
        self.assertIn(self.root, out)
        self.assertIn("/reload-skills", out)
        self.assertIn("Unknown skill", out)

    def test_init_commits_what_it_wrote(self):
        _, out = self.init()
        self.assertIn("закомічено окремо", out)
        self.assertIn("Keel у проєкті", self.log())
        self.assertNotIn("AGENTS.md", self.porcelain())

    def test_no_commit_leaves_it_and_says_so(self):
        _, out = self.init(no_commit=True)
        self.assertNotIn("закомічено", out)
        self.assertIn("не в git", out)
        self.assertIn("git add", out)
        self.assertIn("AGENTS.md", self.porcelain())

    def test_the_commit_message_follows_the_project_language(self):
        self.init(docs=None, lang="en")
        self.assertIn("Keel in the project", self.log())

    def test_a_second_init_adds_no_empty_commit(self):
        self.init()
        before = self.log()
        self.init()
        self.assertEqual(before, self.log())

    def test_the_commit_goes_through_even_with_a_red_hook(self):
        """pre-commit ставиться цим же init; порожній крок його б не пустив."""
        self.write("keel-unrelated.txt", "x")
        code, _ = self.init()
        self.assertEqual(code, 0)
        self.assertIn("Keel", self.log())

    def test_a_repository_without_an_identity_falls_back_to_the_hint(self):
        subprocess.run(["git", "-C", self.root, "config", "user.email", ""], check=True)
        subprocess.run(["git", "-C", self.root, "config", "user.name", ""], check=True)
        _, out = self.init()
        if "закомічено" not in out:
            self.assertIn("не в git", out)

    def test_creates_the_three_folders(self):
        self.init()
        for folder in keel.INIT_DIRS:
            self.assertTrue(os.path.isdir(os.path.join(self.root, folder)), folder)

    def test_vendors_the_tool_verbatim(self):
        self.init()
        with open(keel.__file__, encoding="utf-8") as handle:
            self.assertEqual(self.read(keel.VENDORED), handle.read())

    def test_vendored_tool_runs(self):
        self.init()
        done = subprocess.run(
            [sys.executable, os.path.join(self.root, keel.VENDORED), "check", "--fast"],
            cwd=self.root, capture_output=True, text=True)
        self.assertIn("посилання ведуть кудись", done.stdout)

    def test_copies_every_reference(self):
        self.init()
        self.assertIn("ISO/IEC 25010", self.read("keel/QUALITY.md"))
        self.assertIn("шість перевірок", self.read("keel/KEEL.md"))
        self.assertIn("keel new step", self.read("keel/README.md"))

    def test_methodology_and_tool_stay_apart(self):
        """Розділено навмисно: KEEL.md — що і чому, README.md — чим запускати."""
        self.init()
        method, tool = self.read("keel/KEEL.md"), self.read("keel/README.md")
        self.assertNotIn("| `keel new step", method)
        self.assertNotIn("mix.exs", method)
        self.assertIn("README.md", method)
        self.assertIn("KEEL.md", tool)

    def test_agents_block_points_at_both_references(self):
        self.init()
        block = self.read("AGENTS.md")
        for name in keel.REFERENCES:
            self.assertIn(f"keel/{name}", block)

    def test_agents_block_holds_seven_principles(self):
        self.init()
        block = self.read("AGENTS.md")
        self.assertIn(keel.AGENTS_START, block)
        self.assertIn(keel.AGENTS_END, block)
        self.assertIn("7. Не заводь сутність", block)
        self.assertIn("1. Обіцянка перевіряється", block)

    def test_existing_agents_file_is_kept(self):
        self.write("AGENTS.md", "# Проєкт\n\nСвоє, написане руками.\n")
        self.init()
        text = self.read("AGENTS.md")
        self.assertIn("Своє, написане руками.", text)
        self.assertIn(keel.AGENTS_START, text)

    def test_second_init_replaces_only_the_block(self):
        self.write("AGENTS.md", "# Проєкт\n\nСвоє.\n")
        self.init()
        self.write("AGENTS.md", self.read("AGENTS.md") + "\nДописане після.\n")
        self.init()
        text = self.read("AGENTS.md")
        self.assertEqual(text.count(keel.AGENTS_START), 1)
        self.assertIn("Своє.", text)
        self.assertIn("Дописане після.", text)

    def test_ci_names_the_vendored_tool_and_the_language(self):
        self.init()
        workflow = self.read(keel.CI_FILE)
        self.assertIn("python3 keel/keel.py check", workflow)
        self.assertIn("erlef/setup-beam", workflow)
        self.assertIn("fetch-depth: 0", workflow)

    def test_ci_passes_the_branch_name(self):
        self.init()
        self.assertIn("--branch \"${{ github.head_ref || github.ref_name }}\"",
                      self.read(keel.CI_FILE))

    def test_installs_hooks(self):
        self.init()
        self.assertTrue(os.access(os.path.join(self.root, ".git/hooks/pre-commit"), os.X_OK))

    def test_second_init_writes_nothing_new(self):
        self.init()
        _, out = self.init()
        written = [row for row in out.splitlines() if row.startswith("  ")]
        self.assertNotIn("  AGENTS.md (блок між маркерами)", written)
        self.assertNotIn(f"  {keel.CI_FILE}", written)


# ─────────────────────────────────────────────────────────────────────────────
# keel skills
# ─────────────────────────────────────────────────────────────────────────────




class TestLanguageSettings(unittest.TestCase):
    """Дві мови, і вони незалежні: довідники окремо, мова проєкту окремо."""

    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-lang-")
        self.addCleanup(shutil.rmtree, self.root, True)
        subprocess.run(["git", "init", "-b", "main", "-q", self.root], check=True)

    def init(self, **kwargs):
        from io import StringIO
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            keel.cmd_init(keel.Project(self.root), Args(
                **{"install": True, "force": False, "docs": None,
                   "lang": None, **kwargs}))
        finally:
            sys.stdout = saved
        return stream.getvalue()

    def config(self):
        with open(os.path.join(self.root, keel.CONFIG_FILE), encoding="utf-8") as h:
            return json.load(h)

    def read(self, name):
        with open(os.path.join(self.root, name), encoding="utf-8") as handle:
            return handle.read()

    def test_defaults_are_written_down(self):
        self.init()
        stored = self.config()
        self.assertEqual({k: stored[k] for k in keel.DEFAULTS}, keel.DEFAULTS)
        self.assertIn(keel.VENDORED, stored["generated"])

    def test_settings_are_independent(self):
        """Довідники однією мовою, тригери іншою — саме той випадок."""
        self.init(lang="en")
        self.assertEqual(self.config()["lang"], "en")
        self.assertEqual(self.config()["docs"], keel.SOURCE_LANG)

    def test_language_changes_the_trigger_phrases(self):
        self.init(lang="uk")
        self.assertIn("«зроби наступне»",
                      self.read(".claude/skills/keel-work/SKILL.md"))
        self.init(lang="en")
        text = self.read(".claude/skills/keel-work/SKILL.md")
        self.assertIn("what's next", text)
        self.assertNotIn("«зроби наступне»", text)

    def test_the_body_never_changes_with_language(self):
        """Тіло — інструкція моделі, воно англійською завжди."""
        self.init(lang="uk")
        _, first, _ = keel.split_front_matter(
            self.read(".claude/skills/keel-plan/SKILL.md"))
        self.init(lang="en")
        _, second, _ = keel.split_front_matter(
            self.read(".claude/skills/keel-plan/SKILL.md"))
        self.assertEqual(first, second)

    def test_setting_survives_a_plain_rerun(self):
        self.init(lang="en")
        self.init()
        self.assertEqual(self.config()["lang"], "en")

    def test_english_docs_arrive_in_english(self):
        self.init(docs="en")
        self.assertIn("two kinds of document", self.read("keel/KEEL.md"))
        self.assertIn("A promise is checked", self.read("AGENTS.md"))

    def test_docs_and_lang_really_are_independent(self):
        """Англійський довідник із українськими тригерами — той самий випадок."""
        self.init(docs="en", lang="uk")
        self.assertIn("two kinds of document", self.read("keel/KEEL.md"))
        self.assertIn("«зроби наступне»",
                      self.read(".claude/skills/keel-work/SKILL.md"))

    def test_a_language_without_translations_refuses(self):
        with unittest.mock.patch.object(keel, "doc_source",
                                        lambda name, lang: "/no/such/" + name):
            with self.assertRaises(SystemExit):
                self.init(docs="en")

    def test_a_broken_config_is_not_overwritten(self):
        """Перезапис тихо скинув би мови й перепородив скіли чужими тригерами."""
        self.init(lang="uk")
        os.makedirs(os.path.join(self.root, "keel"), exist_ok=True)
        with open(os.path.join(self.root, keel.CONFIG_FILE), "w") as handle:
            handle.write("{ побите")
        from io import StringIO
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            keel.write_config(self.root, keel.DEFAULTS, [], {})
        finally:
            sys.stdout = saved
        self.assertIn("не чіпаю", stream.getvalue())
        with open(os.path.join(self.root, keel.CONFIG_FILE)) as handle:
            self.assertEqual(handle.read(), "{ побите")

    def test_broken_config_falls_back_to_defaults(self):
        os.makedirs(os.path.join(self.root, "keel"), exist_ok=True)
        with open(os.path.join(self.root, keel.CONFIG_FILE), "w") as handle:
            handle.write("{ not json")
        self.assertEqual(keel.read_config(self.root), keel.DEFAULTS)




class TestUpdate(unittest.TestCase):
    """Оновлення має відрізняти «методика поїхала» від «правили руками»."""

    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="keel-upd-")
        self.addCleanup(shutil.rmtree, self.root, True)
        subprocess.run(["git", "init", "-b", "main", "-q", self.root], check=True)
        self.run_command(keel.cmd_init, install=True, force=False,
                         docs=None, lang=None)

    def run_command(self, command, **kwargs):
        from io import StringIO
        stream, saved = StringIO(), sys.stdout
        sys.stdout = stream
        try:
            code = command(keel.Project(self.root), Args(**kwargs))
        finally:
            sys.stdout = saved
        return code, stream.getvalue()

    def update(self, **kwargs):
        return self.run_command(keel.cmd_update,
                                **{"diff": False, "force": False, **kwargs})

    def path(self, name):
        return os.path.join(self.root, name)

    def read(self, name):
        with open(self.path(name), encoding="utf-8") as handle:
            return handle.read()

    def write(self, name, text):
        with open(self.path(name), "w", encoding="utf-8") as handle:
            handle.write(text)

    SKILL = ".cursor/skills/keel-work/SKILL.md"

    def test_right_after_init_there_is_nothing_to_do(self):
        code, out = self.update()
        self.assertEqual(code, 0)
        self.assertIn("усе на місці", out)

    def test_a_missing_file_is_restored(self):
        os.remove(self.path(self.SKILL))
        code, out = self.update()
        self.assertEqual(code, 0)
        self.assertIn(self.SKILL, out)
        self.assertTrue(os.path.exists(self.path(self.SKILL)))

    def test_a_hand_edit_is_refused_not_clobbered(self):
        self.write(self.SKILL, "правлено руками\n")
        code, out = self.update()
        self.assertEqual(code, 1)
        self.assertIn("правлено руками, не чіпаю", out)
        self.assertEqual(self.read(self.SKILL), "правлено руками\n")

    def test_force_overwrites_a_hand_edit(self):
        self.write(self.SKILL, "правлено руками\n")
        code, _ = self.update(force=True)
        self.assertEqual(code, 0)
        self.assertIn("Generated by", self.read(self.SKILL))

    def test_a_stale_copy_is_refreshed_without_force(self):
        """Файл не чіпали руками, змінилась методика — оновлюємо мовчки."""
        manifest = keel.read_manifest(self.root)
        self.write(self.SKILL, "старе покоління\n")
        manifest[self.SKILL] = keel.digest("старе покоління\n")
        settings = keel.read_config(self.root)
        keel.write_config(self.root, settings, [], manifest)
        code, out = self.update()
        self.assertEqual(code, 0)
        self.assertIn(self.SKILL, out)
        self.assertIn("Generated by", self.read(self.SKILL))

    def test_a_lost_manifest_refuses_instead_of_overwriting(self):
        """Немає запису — невідомо, чи файл правили. Безпечне за замовчуванням."""
        settings = keel.read_config(self.root)
        self.write(self.SKILL, "правлено руками\n")
        keel.write_config(self.root, settings, [], {})
        code, out = self.update()
        self.assertEqual(code, 1)
        self.assertIn("не чіпаю", out)
        self.assertEqual(self.read(self.SKILL), "правлено руками\n")

    def test_the_manifest_heals_itself(self):
        settings = keel.read_config(self.root)
        keel.write_config(self.root, settings, [], {})
        self.update()
        self.assertIn(self.SKILL, keel.read_manifest(self.root))

    def test_running_from_a_vendored_copy_refuses_instead_of_no_op(self):
        """Копія в проєкті порівнювала б кожне джерело саме з собою."""
        with unittest.mock.patch.object(keel, "home", lambda: self.root):
            with self.assertRaises(SystemExit):
                self.update()

    def test_diff_shows_and_changes_nothing(self):
        self.write(self.SKILL, "правлено руками\n")
        code, out = self.update(diff=True)
        self.assertEqual(code, 0)
        self.assertIn("---", out)
        self.assertEqual(self.read(self.SKILL), "правлено руками\n")

    def test_language_change_flows_through_update(self):
        settings = keel.read_config(self.root)
        settings["lang"] = "en"
        keel.write_config(self.root, settings, [], keel.read_manifest(self.root))
        code, _ = self.update()
        self.assertEqual(code, 0)
        self.assertIn("what's next", self.read(self.SKILL))




class TestTranslationsKeepUp(unittest.TestCase):
    """Справжня охорона двомовності.

    `keel update` звіряє переклади лише коли його запустили з дому методики —
    вендорена копія в проєкті джерел поруч не має. Тобто покладатись на неї
    означало б покладатись на те, що хтось згадає. Цей тест біжить щоразу.
    """

    def sources(self):
        return keel.REFERENCES + ("PRINCIPLES.md",)

    def test_every_reference_has_a_translation(self):
        found = keel.translations("en")
        for name in self.sources():
            self.assertIn(name, found, f"немає docs/en/{name}")

    def test_no_translation_has_fallen_behind(self):
        for name, recorded in sorted(keel.translations("en").items()):
            text = keel.read_text(os.path.join(keel.home(), name))
            self.assertTrue(
                keel.rev_matches(recorded, text),
                f"docs/en/{name} тримає {recorded}, а {name} зараз "
                f"{keel.revision(text)}. Перечитай переклад і онови source-rev.")

    def test_translation_carries_the_name_of_its_source(self):
        for name in self.sources():
            path = keel.doc_source(name, "en")
            front, _, _ = keel.split_front_matter(keel.read_text(path))
            self.assertEqual(keel.parse_yaml(front).get("translates"), name)

    def test_the_translated_principles_still_number_seven(self):
        self.assertEqual(len(keel.principles_lines("en")), 7)
        self.assertEqual(len(keel.principles_lines("uk")), 7)

    def test_nothing_ukrainian_was_left_behind(self):
        """Кирилиця в перекладі — це шматок, який забули перекласти."""
        import re
        for name in self.sources():
            text = keel.read_text(keel.doc_source(name, "en"))
            left = re.findall(r"[а-яіїєґА-ЯІЇЄҐ][а-яіїєґ'’ -]{3,}", text)
            self.assertEqual(left, [], f"docs/en/{name}: {left[:3]}")

    def test_the_project_gets_prose_without_the_bookkeeping(self):
        """`source-rev` — облік удома; у проєкт має доїхати сам текст."""
        text = keel.read_text(keel.doc_source("KEEL.md", "en"))
        self.assertIn("source-rev", text)
        self.assertNotIn("source-rev", keel.strip_front_matter(text))


class TestTranslationCheck(unittest.TestCase):
    """Хто спирається — той тримає редакцію. Переклад спирається на джерело."""

    def test_source_language_has_nothing_to_check(self):
        root = tempfile.mkdtemp(prefix="keel-tr-")
        self.addCleanup(shutil.rmtree, root, True)
        os.makedirs(os.path.join(root, "keel/steps"))
        project = keel.Project(root)
        project.settings = {"docs": keel.SOURCE_LANG, "lang": keel.SOURCE_LANG}
        self.assertEqual(keel.check_translations(project), [])

    def test_stale_translation_is_reported(self):
        root = tempfile.mkdtemp(prefix="keel-tr-")
        self.addCleanup(shutil.rmtree, root, True)
        os.makedirs(os.path.join(root, "keel/steps"))
        project = keel.Project(root)
        project.settings = {"docs": "en", "lang": "en"}
        found = {name: "deadbe" for name in keel.REFERENCES}
        with unittest.mock.patch.object(keel, "translations", lambda lang: found):
            problems = keel.check_translations(project)
        self.assertEqual(len(problems), len(keel.REFERENCES))
        self.assertIn("тримає deadbe", problems[0].message)

    def test_translation_without_a_recorded_revision(self):
        root = tempfile.mkdtemp(prefix="keel-tr-")
        self.addCleanup(shutil.rmtree, root, True)
        os.makedirs(os.path.join(root, "keel/steps"))
        project = keel.Project(root)
        project.settings = {"docs": "en", "lang": "en"}
        with unittest.mock.patch.object(keel, "translations",
                                        lambda lang: {"KEEL.md": ""}):
            problems = keel.check_translations(project)
        self.assertIn("не називає редакції", problems[0].message)


if __name__ == "__main__":
    unittest.main()
