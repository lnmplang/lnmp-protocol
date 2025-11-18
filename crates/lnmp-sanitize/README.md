# lnmp-sanitize

Lenient sanitizer for LNMP text inputs. This crate performs lightweight whitespace cleanup,
escape repair, and basic normalization so that downstream strict parsers can accept
slightly malformed inputs from LLMs or user-written payloads.

The API is intentionally streaming-friendly and returns a `Cow<'a, str>` to avoid
unnecessary allocations when the input already matches the expected format.
