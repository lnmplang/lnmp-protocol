#!/usr/bin/env python3
"""
Real OpenAI token counter using tiktoken.

Usage:
  echo "Your text here" | python3 count_tokens.py
  python3 count_tokens.py --model gpt-4o < input.txt
"""

import sys
import argparse

try:
    import tiktoken
except ImportError:
    print("ERROR: tiktoken not installed. Run: pip install tiktoken", file=sys.stderr)
    sys.exit(1)

def count_tokens(text: str, model: str = "gpt-4o") -> int:
    """Count tokens using OpenAI's tiktoken library."""
    try:
        enc = tiktoken.encoding_for_model(model)
        return len(enc.encode(text))
    except KeyError:
        # Fallback to cl100k_base (GPT-4 encoding)
        enc = tiktoken.get_encoding("cl100k_base")
        return len(enc.encode(text))

def main():
    parser = argparse.ArgumentParser(description="Count OpenAI tokens using tiktoken")
    parser.add_argument("--model", default="gpt-4o", help="OpenAI model name (default: gpt-4o)")
    parser.add_argument("--verbose", "-v", action="store_true", help="Show detailed info")
    args = parser.parse_args()
    
    # Read from stdin
    text = sys.stdin.read()
    
    if not text.strip():
        print("ERROR: No input text provided", file=sys.stderr)
        sys.exit(1)
    
    # Count tokens
    token_count = count_tokens(text, args.model)
    
    if args.verbose:
        print(f"Model: {args.model}", file=sys.stderr)
        print(f"Text length: {len(text)} chars", file=sys.stderr)
        print(f"Token count: {token_count}", file=sys.stderr)
        print(f"Chars/token: {len(text)/token_count:.2f}", file=sys.stderr)
        print()
    
    # Output just the number (for easy parsing)
    print(token_count)

if __name__ == "__main__":
    main()
