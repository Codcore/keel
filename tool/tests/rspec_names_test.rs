//! Scenario test of wave 0047: a spec's name is what rspec calls it.
//!
//! The tag reader, over `_spec.rb` files, composes rspec's FULL
//! DESCRIPTION out of the groups above the example -- and the same
//! names come back from a real `rspec --dry-run`, which is what the
//! adapter selects an example by. Two examples no tag can name are
//! refusals that say which.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use std::path::Path;
use std::process::Command;

/// The forms rspec's own documentation writes, nested as people nest
/// them: a constant, a string, `#method`, `.method`, `::CONST`,
/// metadata after the name, `it`/`specify`/`example`, two kinds of
/// quotes with escapes, a second argument to `describe`, a symbol, a
/// `described_class`, a bare `describe` outside `RSpec.` -- and the
/// ruby BETWEEN the groups that review 0047 R-2 found the reader
/// counting as groups' ends: a `def`, an `if`, a `case`, a heredoc,
/// a trailing comment after `do`, a block with `|ex|`.
const SPEC: &str = "require \"spec_helper\"

value = \"here\"

RSpec.describe Toy do
  # proves: it-works@aaaaaa
  it \"works\" do
  end

  describe \"#works\" do
    context \"when called twice\" do
      # proves: it-works@aaaaaa
      it \"still returns true\" do
      end
    end
  end

  describe \".build\" do
    # proves: it-works@aaaaaa
    specify 'builds' do
    end
  end

  context \"with metadata\", :slow do
    # proves: it-works@aaaaaa
    example \"has metadata\", :fast, focus: false do
    end
  end

  describe Toy, \"with a second arg\" do
    # proves: it-works@aaaaaa
    it \"two args\" do
    end
  end

  context \"when called\" do
    describe \"#works\" do
      # proves: it-works@aaaaaa
      it \"returns true\" do
      end
    end
  end

  # proves: it-works@aaaaaa
  it \"#works\" do
  end

  describe \"::VERSION\" do
    # proves: it-works@aaaaaa
    it \"exists\" do
    end
  end

  describe :works do
    # proves: it-works@aaaaaa
    it \"answers\" do
    end
  end

  describe described_class do
    # proves: it-works@aaaaaa
    it \"works\" do
    end
  end

  # proves: it-works@aaaaaa
  it \"escaped \\\"quote\\\"\" do
  end

  # proves: it-works@aaaaaa
  it \"takes the example\" do |ex|
  end

  def helper
    1
  end

  # proves: it-works@aaaaaa
  it \"after def\" do
  end

  before do
    if true
      x = 1
    end
    y = case x when 1 then 2 else 3 end
    @text = <<~TXT
      end
      end
      it \"ghost\" do
    TXT
  end

  # proves: it-works@aaaaaa
  it \"after if, case and heredoc\" do
  end

  it \"other\" do # a note
  end

  # proves: it-works@aaaaaa
  it \"after a comment\" do
  end

  describe Toy, \"#works\" do
    # proves: it-works@aaaaaa
    it \"returns true\" do
    end
  end

  describe \"a\", \"#b\" do
    # proves: it-works@aaaaaa
    it \"c\" do
    end
  end
end

RSpec.describe \"Toy\" do
  describe \"#works\" do
    # proves: it-works@aaaaaa
    it \"returns true\" do
    end
  end
end

describe \"bare group\" do
  # proves: it-works@aaaaaa
  it \"outside RSpec\" do
  end
end
";

/// What rspec calls each tagged example above, in order -- measured
/// with `rspec --dry-run --format json`, and checked against it again
/// below. The rule of the join is rspec's own: no space only after a
/// MODULE constant before `#…`, `::…` or `.…`; the example's own name
/// always takes a space.
const NAMES: [&str; 19] = [
    "Toy works",
    "Toy#works when called twice still returns true",
    "Toy.build builds",
    "Toy with metadata has metadata",
    "Toy Toy with a second arg two args",
    "Toy when called #works returns true",
    "Toy #works",
    "Toy::VERSION exists",
    "Toy works answers",
    "Toy Toy works",
    "Toy escaped \"quote\"",
    "Toy takes the example",
    "Toy after def",
    "Toy after if, case and heredoc",
    "Toy after a comment",
    "Toy Toy#works returns true",
    "Toy a #b c",
    "Toy #works returns true",
    "bare group outside RSpec",
];

/// proves: a-spec-name-is-what-rspec-calls-it@ee389e
#[test]
fn a_spec_name_is_what_rspec_calls_it() {
    // The reader composes the full description: groups joined by a
    // space, no space before `#…` and `.…`, a constant by its name,
    // metadata after the name left out.
    let tags = match keel::tags::scan_text(Path::new("spec/toy_spec.rb"), SPEC) {
        Ok(tags) => tags,
        Err(err) => panic!(
            "every tag over an example is read: {} / {}",
            err.reason, err.instead
        ),
    };
    let names: Vec<&str> = tags.iter().map(|tag| tag.test.as_str()).collect();
    assert_eq!(
        names,
        NAMES.to_vec(),
        "the names are rspec's own full descriptions"
    );

    // A one-liner `it { … }` has no name rspec would call it by
    // (`example at ./spec/…:N`): a tag over it is refused as one.
    let one_liner =
        "RSpec.describe Toy do\n  # proves: it-works@aaaaaa\n  it { expect(1).to eq(1) }\nend\n";
    let Err(err) = keel::tags::scan_text(Path::new("spec/toy_spec.rb"), one_liner) else {
        panic!("a one-liner example has no name for a tag to hold");
    };
    assert!(
        err.reason.contains("it {"),
        "and the refusal names the one-liner: {}",
        err.reason
    );

    // A name rspec builds at run time, and a group named by something
    // the reader cannot read -- a value constant, a variable -- are
    // refusals that say which (review 0047 R-6): "did not run" over
    // a name nobody wrote is not an answer.
    let dynamic = "RSpec.describe Toy do\n  value = 1\n  # proves: it-works@aaaaaa\n  it \"works #{value}\" do\n  end\nend\n";
    let Err(err) = keel::tags::scan_text(Path::new("spec/toy_spec.rb"), dynamic) else {
        panic!("a name built at run time cannot be read from the source");
    };
    assert!(
        err.reason.contains("#{"),
        "and the refusal names the interpolation: {}",
        err.reason
    );
    for (line, what) in [
        ("describe Toy::VERSION do", "Toy::VERSION"),
        ("describe subject_class do", "subject_class"),
        ("describe some.method do", "some.method"),
    ] {
        let text = format!(
            "RSpec.describe Toy do\n  {line}\n    # proves: it-works@aaaaaa\n    it \"works\" do\n    end\n  end\nend\n"
        );
        let Err(err) = keel::tags::scan_text(Path::new("spec/toy_spec.rb"), &text) else {
            panic!("{line}: a group the reader cannot name is refused, not guessed");
        };
        assert!(
            err.reason.contains(what),
            "{line}: and the refusal names the argument: {}",
            err.reason
        );
    }

    // An example with no name at all -- `it { … }` and `it do … end`
    // alike (review 0047 R-9) -- is refused as one.
    let it_do = "RSpec.describe Toy do\n  # proves: it-works@aaaaaa\n  it do\n    expect(1).to eq(1)\n  end\nend\n";
    let Err(err) = keel::tags::scan_text(Path::new("spec/toy_spec.rb"), it_do) else {
        panic!("an example without a name has no name for a tag to hold");
    };
    assert!(
        err.reason.contains("it do"),
        "and the refusal names that form too: {}",
        err.reason
    );

    // An example inside `shared_examples` has as many names as the
    // places that include it: refused as one, with the parent named.
    let shared = "RSpec.describe Toy do\n  shared_examples \"shared\" do\n    # proves: it-works@aaaaaa\n    it \"is shared\" do\n    end\n  end\n\n  it_behaves_like \"shared\"\nend\n";
    let Err(err) = keel::tags::scan_text(Path::new("spec/toy_spec.rb"), shared) else {
        panic!("an example inside shared_examples has no single name");
    };
    assert!(
        err.reason.contains("shared_examples"),
        "and the refusal names shared_examples: {}",
        err.reason
    );

    // And a `_test.rb` file is still minitest's: `def test_…` named
    // by the method, no groups.
    let minitest = "class ToyTest < Minitest::Test\n  # proves: it-works@aaaaaa\n  def test_it_works\n  end\nend\n";
    let tags = keel::tags::scan_text(Path::new("test/toy_test.rb"), minitest)
        .unwrap_or_else(|err| panic!("a minitest tag is read as before: {}", err.reason));
    assert_eq!(tags[0].test, "test_it_works");

    // The same names from rspec itself: a real dry run over the same
    // file reports exactly these full descriptions -- which is the
    // list the adapter selects an id from.
    if !common::machine_has("rspec").ready() {
        return;
    }
    let dir = common::sandbox("rsnames");
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::create_dir_all(dir.join("spec")).unwrap();
    std::fs::write(
        dir.join("lib/toy.rb"),
        "module Toy\n  VERSION = \"1\"\nend\n",
    )
    .unwrap();
    std::fs::write(dir.join(".rspec"), "--require spec_helper\n").unwrap();
    std::fs::write(dir.join("spec/spec_helper.rb"), "require \"toy\"\n").unwrap();
    std::fs::write(dir.join("spec/toy_spec.rb"), SPEC).unwrap();
    let out = Command::new("rspec")
        .args([
            "--dry-run",
            "--format",
            "json",
            "--no-color",
            "spec/toy_spec.rb",
        ])
        .current_dir(&dir)
        .env_remove("SPEC_OPTS")
        .output()
        .unwrap();
    let said = String::from_utf8_lossy(&out.stdout).into_owned();
    for name in NAMES {
        let json_name = name.replace('"', "\\\"");
        assert!(
            said.contains(&format!("\"full_description\":\"{json_name}\"")),
            "rspec itself calls the example {name:?}:\n{said}"
        );
    }
    assert!(
        !said.contains("ghost"),
        "and the heredoc's `it` is text to rspec as to the reader:\n{said}"
    );
}
