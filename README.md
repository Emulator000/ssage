# Ssage

### Extract important words from a sequence of messages or a full text and give it singular priority.

## Sample of use

Supposing a sentence like `"hi! this is just a sample message with distinct words."` we could extract keywords and prioritize it.

```rust
use ssage::Ssage;

fn sample() {
    let mut ssage = Ssage::new(Default::default());

    let _ = ssage.feed("hi! this is just a sample message with distinct words.");
    ssage.prioritize_keyword("message");
    ssage.prioritize_keyword("just");
    ssage.prioritize_keyword("just");
    ssage.prioritize_keyword("message");
    ssage.prioritize_keyword("message");
    ssage.prioritize_keyword("just");
    ssage.prioritize_keyword("message");

    println!("Output: {}", ssage.feed("just a message"));
}
```

This should output `message just` since we prioritize more `message` than `just` and as the default configuration excluded words that are less than 4 characters.
