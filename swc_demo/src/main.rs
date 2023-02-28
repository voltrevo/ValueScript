use std::sync::Arc;
use swc_common::{
  errors::{ColorConfig, Handler},
  FileName, SourceMap,
};
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::{Syntax, TsConfig};

fn main() {
  let cm = Arc::<SourceMap>::default();
  let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));
  let c = swc::Compiler::new(cm.clone());
  // let fm = cm
  //     .load_file(Path::new("foo.js"))
  //     .expect("failed to load file");
  let fm = cm.new_source_file(
    FileName::Custom("test.js".into()),
    "
      if (x < 3) {
        1 + 1;
      }
    "
    .into(),
  );
  let result = c.parse_js(
    fm,
    &handler,
    EsVersion::Es2020,
    Syntax::Typescript(TsConfig::default()),
    swc::config::IsModule::Bool(true),
    None,
  );

  dbg!(&result);

  drop(result);
}
