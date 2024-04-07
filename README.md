# 1BRC in Rust

My attempt at the [One Billion Row Challenge](https://github.com/gunnarmorling/1brc) in Rust. The only dependency used so far is a 
crate to replace the standard library's `HashMap` since it is relatively slow, and tuned for 
security.

Runs in about 6-7 seconds on my M1 Macbook Pro in release mode.

## License

[MIT](LICENSE)
