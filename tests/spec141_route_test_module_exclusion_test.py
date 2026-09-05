#!/usr/bin/env python3
"""Source inventory excludes cfg(test) fixtures, not later production items."""
import importlib.util
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location(
    "route_classification", ROOT / "scripts/generate-agent-route-classification.py"
)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class TestModuleExclusion(unittest.TestCase):
    def test_nested_module_literals_and_following_production(self):
        body = r'''
fn before() { router.route("/before", get(handler)); }
#[cfg(test)]
mod tests {
    const RAW: &str = r##"} /* not comment */ { "##;
    const BYTE_RAW: &[u8] = br#"}"#;
    const TEXT: &str = "escaped \" }";
    const CHARACTER: char = '}';
    /* nested /* } */ { */
    mod nested { fn fixture() { router.route("/test-only", post(handler)); } }
}
fn after() { router.route("/after", post(handler)); }
'''
        result = module.without_inline_test_modules(body)
        self.assertIn('router.route("/before"', result)
        self.assertIn('router.route("/after"', result)
        self.assertNotIn('/test-only', result)
        self.assertEqual(len(result), len(body))
        self.assertEqual(result.count('\n'), body.count('\n'))

    def test_comment_or_literal_attributes_are_not_modules(self):
        body = r'''
// #[cfg(test)] mod pretend { }
const TEXT: &str = r#"#[cfg(test)] mod pretend { }"#;
/* #[cfg(test)] mod pretend { } */
#[cfg(not(test))]
mod production { fn route() { router.route("/real", get(handler)); } }
'''
        self.assertEqual(module.without_inline_test_modules(body), body)

    def test_multiple_modules_and_attributes(self):
        body = '''#[cfg(test)] #[allow(dead_code)] pub(crate) mod first { }
fn production() { router.route("/real", get(handler)); }
#[cfg ( test )] mod second { router.route("/fixture", get(handler)); }
'''
        result = module.without_inline_test_modules(body)
        self.assertNotIn('mod first', result)
        self.assertNotIn('/fixture', result)
        self.assertIn('/real', result)

    def test_unterminated_input_fails_closed(self):
        for body in ['#[cfg(test)] mod broken {', '/* missing terminator']:
            with self.assertRaises(ValueError):
                module.without_inline_test_modules(body)

    def test_actual_auth_fixture_is_not_an_api_surface(self):
        body = (ROOT / 'crates/focusa-api/src/middleware/auth.rs').read_text()
        self.assertIn('/v1/state/permission-probe', body)
        result = module.without_inline_test_modules(body)
        self.assertNotIn('/v1/state/permission-probe', result)
        self.assertIn('pub async fn auth_layer', result)


if __name__ == '__main__':
    unittest.main()
