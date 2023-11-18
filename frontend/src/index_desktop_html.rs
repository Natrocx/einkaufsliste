use const_format::formatcp;

use crate::ui::consts::PRIMARY_BG;

pub static INDEX_HTML: &str = formatcp!(
  "
<!DOCTYPE html>
<html>

<head>
  <title>dioxus | ⛺</title>
  <meta content=\"text/html;charset=utf-8\" http-equiv=\"Content-Type\" \
   />
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
  <meta charset=\"UTF-8\" />

  <link href=\"https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined\" \
   rel=\"stylesheet\" />

  <style>
    {css}
  </style>
</head>

<body>
  <div id=\"main\" class=\"h-screen w-screen {PRIMARY_BG} dark:text-white\"></div>
</body>

</html>",
  css = include_str!("../dist/assets/tailwind.css")
);
