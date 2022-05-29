# tissues
A [Linear](https://linear.app) CLI client to display dashboards of multiple custom views (issues) and allow for fast modification.

<img width="800px" src="https://raw.githubusercontent.com/kgodara/tissues/master/docs/resources/demo.gif" />

## Motivation
[Linear](https://linear.app) doesn't currently support viewing multiple custom views simultaneously.
This project provides a simple Linear client TUI which queries (via the Linear API) issues from custom views, allows the user to display in a customizable dashboard, modify issues (incl. editing content), as well as pagination, caching, and a variety of other helpful improvements.

Tests are included; however, be aware that on most recent verification, certain aspects of the Linear Custom View API remained broken, as confirmed by the Linear team. Bug reports have been submitted, but fixes are a low priority.

## Future Goals

 - Support for Github
 - Support for Workflows
 - Improvements to column layout framework, resize handling, formatting
 - And much more
