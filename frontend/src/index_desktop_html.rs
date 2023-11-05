use const_format::formatcp;

pub static INDEX_HTML: &str = formatcp!(
  "
<!DOCTYPE html>
<html>

<head>
  <title>dioxus | ⛺</title>
  <meta content=\"text/html;charset=utf-8\" http-equiv=\"Content-Type\" />
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
  <meta charset=\"UTF-8\" />

  <link href=\"https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined\" rel=\"stylesheet\" />

  <style>
    {css}
  </style>
</head>

<body>
  <div id=\"main\"></div>
</body>

</html>",
  css = include_str!("../dist/assets/tailwind.css")
);
