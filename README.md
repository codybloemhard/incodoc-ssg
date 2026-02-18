# incodoc-ssg

Static site generator (SSG) using incodoc. Early stages WIP.

Incodoc is an incorporeal document format.
For more see: <https://github.com/codybloemhard/incodoc>

1. should take in markdown and/or incodoc
2. do its thing
3. output html and/or incodoc for deployment

## Features

Features are project to include:

- commit files into production
- manage dates and versions of pages
- manage archived pages that are citeable
- warning for dangling pages
- generates purely static incodoc and html/css pages

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
