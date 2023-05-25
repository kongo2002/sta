# sta

*sta* is a command line utility for quick data analysis that currently supports
output of a text histogram.


## Motivation

Whenever you have some basic input data you want to get a first impression on, I
usually tend to get a histogram or some basic stats like `mean`, `variance` and
`median`. Often I went for `histogram.py` from
[data_hacks](https://github.com/bitly/data_hacks) for that purpose. However the
project is not maintained anymore and is not compatible with recent Python 3
versions anymore.

Now I finally went ahead and wrote this very simple alternative in Rust so I can
easily install (or rather copy a binary) on any machine I need it.


## Running

`sta` expects input from `stdin` and outputs a basic text diagram to `stdout`.


### Basic 1-dimensional values

```console
$ sta -b 5 <<EOF
1
4
2
2
2
EOF
# samples: 5; min: 1; max: 4
# mean: 2.20; var: 0.96; sd: 0.98, median: 2.00
# each * represents a count of 1
1.00 - 1.75 [1] *
1.75 - 2.50 [3] ***
2.50 - 3.25 [0]
3.25 - 4.00 [1] *
```


### Key-value data

Use `-f` (or `--format`) to instruct `sta` to parse key-value formatted data
delimited by whitespace:

```console
$ sta -f kv <<EOF
2 6
1 9
3 4
4 1
5 2
9 1
EOF
# samples: 23; min: 1; max: 9
# mean: 2.52; var: 3.06; sd: 1.75, median: 3.50
# each * represents a count of 1
1.00 - 1.80 [9] *********
1.80 - 2.60 [6] ******
2.60 - 3.40 [4] ****
3.40 - 4.20 [1] *
4.20 - 5.00 [2] **
5.00 - 5.80 [0]
5.80 - 6.60 [0]
6.60 - 7.40 [0]
7.40 - 8.20 [0]
8.20 - 9.00 [1] *
```


## Build

You can use the usual Rust toolchain via `cargo` to build, test and run `sta` by
yourself:


### Compile

```console
$ cargo build --release
```


### Run tests

```console
$ cargo test
```
