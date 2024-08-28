# OreStaty
OreStaty - HTML-centered [handlebars](https://crates.io/crates/handlebars)-based static site generator

## Using it as a library
Look at [main.rs](https://github.com/InfiniteCoder01/orestaty/blob/master/src/main.rs) to see how you can use OreStaty as a library in your Rust projects

## CLI
Subcommands:
* build (assumed by default)
Flags:
* -p/--path - specify project path (current directory by default)
* -o/--output - specify output directory (dist by defalt)

## Directory structure
Only "src" directory is mandatory. All files in it are gonna be built (.html/.htm/.hbs - handlebars, .md/.markdown - markdown + handlebars, .css/.scss/.sass - SASS)
All files in "static" directory are gonna be copied to output directory
In "plugins" directory you can put:
* Handlebars templates for rendering HTML and Markdown
* rhai helper scripts for Handlebars
(see [percent.rhai](https://github.com/InfiniteCoder01/orestaty/blob/master/example/plugins/example/percent.rhai) and
[page.md](https://github.com/InfiniteCoder01/orestaty/blob/master/example/src/page.md); Note: Handlebars helpers have scope in
form of `example_percent`, not `example.percent`. Also, in Markdown you might need to escape quotes in string params)

All files in plugins directory get scope. For example, plugins/theme/template.html will be registered as Handlebars template with name `theme.template`
"plugins" is a load path for SASS (if you put bulma-css into this directory, you can import it with `@import bulma/bulma`).

All files/directories showcased in this example of a directory tree:
```
project_directory
├─src
│  ├─index.html
│  ├─page.md
│  └─global.scss
│
├─static
│  ├─image.png
│  └─robots.txt
│
├─plugins
│  ├─example.rhai
│  └─bulma
│      ├─css
│      ├─sass
│      ├─versions
│      ├─bulma.scss
│      ├─LICENSE
│      ├─package.json
│      └─README.md
│
└─dist (auto generated)
```

## Config
You can configure your site in config.toml
Here is an example showcasing all possible config options:
```toml
default_template = "template"
default_markdown_template = "template"
default_highlight_theme = "template"
```
