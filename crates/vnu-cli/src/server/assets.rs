use std::sync::LazyLock;

pub static INDEX_HTML: LazyLock<String> = LazyLock::new(|| {
    let html = include_str!("../../assets/index.html");
    let css = include_str!("../../assets/style.css");
    let js = include_str!("../../assets/app.js");
    html.replace("/* __STYLE__ */", css)
        .replace("/* __SCRIPT__ */", js)
});
