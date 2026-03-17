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

After having done that, you can optionally run `incodoc-ssg conf rss` to create the RSS feeds.

To add your first page, let's say `index.md`, we run the `add` command:
`incodoc-ssg conf add ~/src/index.md`.

This will add the page to the config file and generate the output.

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
