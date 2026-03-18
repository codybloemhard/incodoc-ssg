# incodoc-ssg

Static site generator (SSG) using incodoc.

Incodoc is an incorporeal document format.
For more see: <https://github.com/codybloemhard/incodoc>

Write pages in incodoc compatible markdown and export it as a minimalistic website.

## Features

- easy to set up
- commit files into production
  - choose which pages will be in the end result
  - remove, enable and disable them easily
- manage dates and versions of pages
  - bump version as you update pages
  - dates and versions are inserted into the documents
- generates purely static pages
  - generates both a HTML+CSS page and an incodoc page
  - HTML `link` points to the alternate incodoc version
- generate RSS feed
  - two feeds: one for HTML+CSS pages and one for incodoc pages
  - feeds updated on page updates
- minimalistic header and footer
  - header with a navigation link and link to incodoc version
  - footer with copyright, version and date
- simple configuration file
- simple CLI interface and coloured output

Features that might come in the future:

- manage archived pages that are citeable
- rendered code blocks (show code with no JS)
- improve internal link handling

## Usage

### Setting up

Simple things to know:

- config file: keeps track of everything
- source: directory where you have your input documents (in markdown)
- destination: directory where the output will be generated

I suppose you have some kind of source directory, e.g. `~/src`.
Make sure you have a destination directory, e.g. `mkdir ~/dst`.

Initiate a config file.
In this example the file is called `conf`.
This config file may be present next to the source, destination or somewhere completely else.
You can generate it like this: `incodoc-ssg conf init`;

It looks like this:

```
src: /some/dir
dst: /another/dir
css: style.css
link: https://website.com
lang: en
author: Firstname Lastname
title: Website of Firstname Lastname.
description: Very interesting stuff.
```

Finish the configuration by replacing the default values with your own.
The parsing is very minimal, so make sure not to change anything.
Keep everything in the same order, just replace the values.

To help with getting things right, there are flags in `init` to immediately set some fields.
These are `--src`, `--dst` and `--css`.
For example: `incodoc-ssg conf init --src ~/src`.
This helps you set these paths correctly by being able to use your shell's autocomplete for paths.

The CSS field requires a path to a file within the destination directory.
So if you have `~/dst` as a destination, and `~/dst/style.css` as a style sheet, the CSS field
needs to be `style.css` and not `~/dst/style.css`.
If you use `incodoc-ssg conf init --dst ~/dst --css ~/dst/css.style` it will take care of it.

After having done that, you can optionally run `incodoc-ssg conf rss` to create the RSS feeds.

To add your first page, let's say `index.md`, we run the `add` command:
`incodoc-ssg conf add ~/src/index.md`.

This will add the page to the config file and generate the output.

### Further use

Now you can add more pages to your website.
You can list all entries in the config with `incodoc-ssg conf list`.
The remaining commands are `remove`, `enable`, `disable` and `update`.
All of these take one argument to a page like this: `incodoc-ssg conf remove ~/dst/page.html`.
When specifying the page, you can give the path to the source file (markdown) or to either the
HTML or the incodoc destination file.
It all does the same, so pick whatever is easiest in your case.
`remove` removes the entry from the list in the config and deletes the files in destination.
`disable` removes the files in destination but leaves the entry in disabled mode.
`enable` enables an entry again and generates the files back into destination.

After having changed a source document, you can run `update` to generate the new version in the
destination directory.
`update` comes with an extra argument: the version bump.
You can choose between `major`, `minor`, `patch` or anything else like `keep`.
The first three bump the respective section of the version by one.
Anything else, like `keep`. keeps the version the same.
Except for `new`.
This is used internally when you use `add`.
It will keep the version the same but it will update the RSS files.
Other than that `major` and `minor` also generate a new item in the RSS files.
`patch` or `keep` do not.

### Document writing

#### Metadata

You can use incodoc metadata.
Here is an example:

```md
+++
prop table-of-contents include

nav
    link up to parent page $ ./../index.html
end
+++

# Document heading

some text...
```

Here we set the property that suggests the use of a table of contents.
We do not dictate the look and behaviour of a document in incodoc, so this is an suggestion.
This SSG program will take that suggestion however when generating HTML pages.
It will also check if there is navigation data, and if there is it will take the first link
and put it up in the header of HTML version.
In the example, we use it to link back up to the top page from one directory down.

#### Links

Local links to .md documents are taken to be links to other pages that will be published.
These links have to start with `./` e.g. `./../other-dir/other-page.md`.
It will take links that start with `./` and replaces the first `.md` with another extension.
This way you can link to `.md` in your writings, but generated HTML pages link to generated
HTML pages and generated incodoc pages link to generated incodoc pages.

### Incodoc versions of pages

The HTML version of pages have small niceties like naming the version and date and author in the
footer, having a header with a link and generating a table of contents when suggested.
In incodoc versions, the data that is used to generate these features is instead plainly put
into the document.
The renderer will decide what it does with that.
This is the spirit of incodoc.
The document consumer will have their preferred way of having (or not) a table of contents,
how to navigate and where to display the author and dates (and in what format).

An example of the incodoc data that is inserted into the generated incodoc:

```
props {
    ("date-rfc2822", "Tue, 17 Mar 2026 18:26:59 +0100"),
    ("author", "Cody Bloemhard"),
    ("date-unix", "1773768419"),
    ("table-of-contents", "include"),
    ("initial-version-date", "2026-03-11 Wed"),
    ("version", "0.3.0"),
    ("lang", "en"),
},
nav {
    link {
        "./../index.html",
        "up to parent page",
    },
},
```
 
## Development

incodoc-ssg is build upon:

- <https://github.com/codybloemhard/incodoc>
- <https://github.com/codybloemhard/md-to-incodoc>
- <https://github.com/codybloemhard/incodoc-to-html>

### Development philosophy

incodoc-ssg is meant to be a simple and stable SSG.
It shall not have a tremendous amount of features. 
Results can be displayed in simple browsers without JS support and incodoc renderers.
Generating pages with some kind of scripting such as a search bar is out of scope.
This project also functions as a example of the incodoc libraries being used in an user
facing project.
You are more than welcome to fork this and add as many cool features as you want or build your
own projecs and products that support incodoc.

## License

Copyright (C) 2026 Cody Bloemhard

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
