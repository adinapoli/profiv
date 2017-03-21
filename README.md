
Profiv (__pro__ file __v__ isualiser) is a console-based GHC .prof file
visualiser and parser written in Rust. **It's currently in alpha stage.**

![barebone](https://cloud.githubusercontent.com/assets/442035/24157921/16b04a9c-0e5b-11e7-8e08-da5b64d9f091.gif)

## Why?

Flame-graphs or other fancy ways to navigate the GHC prof files confuses me. All I wanted was the
plain old .prof file, but on steroid.

## TODO

- [X] Parsing of `ExtendedSummaryLine` into a Rose Tree
- [ ] Better error reporting in the parser
- [ ] Ability to collapse each `Forest`
- [ ] Ability to diff 2 .prof files
- [ ] Ability to scroll and lazy-loading of the summary lines according to the viewport dimensions

## Disclaimer

I reserve the right of get bored and do something which actually makes money.
