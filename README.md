
Profiv (__pro__file __v__isualiser) is a console-based GHC .prof file
visualiser and parser written in Rust. **It's currently in alpha stage.**

![screen](https://cloud.githubusercontent.com/assets/442035/21699454/b62c8cdc-d393-11e6-8d2a-af57bbaf659e.png)

## Why?

Flame-graphs or other fancy ways to navigate the GHC prof files confuses me. All I wanted was the
plain old .prof file, but on steroid.

## TODO

- [ ] Parsing of `ExtendedSummaryLine` into a Rose Tree
- [ ] Better error reporting in the parser
- [ ] Ability to collapse each `Forest`
- [ ] Ability to diff 2 .prof files
- [ ] Ability to scroll and lazy-loading of the summary lines according to the viewport dimensions

## Disclaimer

I reserve the right of get bored and do something which actually makes money.
