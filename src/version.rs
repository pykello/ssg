pub const FEATURES: &str = "geomdsl,learning,math-shorthand,proof-directives";

pub const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (git ",
    env!("SSG_GIT_SHA"),
    ", built ",
    env!("SSG_BUILD_DATE"),
    ", features: ",
    "geomdsl,learning,math-shorthand,proof-directives",
    ")"
);
