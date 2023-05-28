# sta

*sta* is a command line utility for quick data analysis that currently supports
output of a text histogram.

[![sta](https://github.com/kongo2002/sta/actions/workflows/build.yml/badge.svg)][actions]


## Motivation

Whenever you have some basic input data you want to get a first impression on, I
usually tend to get a histogram or some basic stats like `mean`, `variance` and
`median`. Often I went for `histogram.py` from
[data_hacks](https://github.com/bitly/data_hacks) for that purpose. However the
project is not maintained anymore and is not compatible with recent Python 3
versions anymore.

Now I finally went ahead and wrote this very simple alternative in Rust so I can
easily install (or rather copy a binary) on any machine I need it.


## Install

Go to the [releases page][releases], expand the list of assets and download a
ready-to-run binary.


## Usage


### Histogram

`sta` expects input from `stdin` and its `histogram` command print a histogram
of the input values.


#### Basic 1-dimensional values

```console
$ sta histogram -b 5 <<EOF
1
4
2
2
2
# samples: 5; min: 1; max: 4
# mean: 2.20; var: 0.96; sd: 0.98, median: 2.00
# each ∎ represents a count of 1
1.000 - 1.600 [1] ∎
1.600 - 2.200 [3] ∎∎∎
2.200 - 2.800 [0]
2.800 - 3.400 [0]
3.400 - 4.000 [1] ∎
```


#### Key-value data

Use `-f` (or `--format`) to instruct `sta` to parse key-value formatted data
delimited by whitespace:

```console
$ sta histogram -f kv <<EOF
2 6
1 9
3 4
4 1
5 2
9 1
EOF
# samples: 23; min: 1; max: 9
# mean: 2.52; var: 3.06; sd: 1.75, median: 3.50
# each ∎ represents a count of 1
1.00 - 1.80 [9] ∎∎∎∎∎∎∎∎∎
1.80 - 2.60 [6] ∎∎∎∎∎∎
2.60 - 3.40 [4] ∎∎∎∎
3.40 - 4.20 [1] ∎
4.20 - 5.00 [2] ∎∎
5.00 - 5.80 [0]
5.80 - 6.60 [0]
6.60 - 7.40 [0]
7.40 - 8.20 [0]
8.20 - 9.00 [1] ∎
```

#### Logarithmic scale

You can also output the histogram values on a log scale:

```console
$ echo 'import random\nfor i in range(1000):\n print(random.randint(0,10000))' \
  | python3 \
  | sta histogram --log
# samples: 1000; min: 2; max: 9987
# mean: 5134.01; var: 8482951.15; sd: 2912.55, median: 5065.50
# each ∎ represents a count of 10
   2.00 -   11.76 [  1]
  11.76 -   31.28 [  2]
  31.28 -   70.32 [  4]
  70.32 -  148.41 [  5]
 148.41 -  304.58 [ 22] ∎∎
 304.58 -  616.91 [ 34] ∎∎∎
 616.91 - 1241.58 [ 49] ∎∎∎∎
1241.58 - 2490.93 [117] ∎∎∎∎∎∎∎∎∎∎∎
2490.93 - 4989.62 [254] ∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
4989.62 - 9987.00 [512] ∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
```


### Bar diagram

You can also print a bar diagram based on the input data on `stdin` via the
`bar` command. Paired with the value-key format (`--format vk`) this can be very
useful for output of `uniq -c`:

```console
$ grep -o '[[:alpha:]]*' src/main.rs \
  | sort | uniq -c | sort -n | tail \
  | sta bar -f vk
# each ∎ represents a count of 1
buckets [26] ∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
    err [37] ∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
    let [58] ∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
  count [29] ∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
    idx [19] ∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
    max [23] ∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
      f [29] ∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
   line [23] ∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
 bucket [26] ∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
     if [22] ∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎∎
```


### Percentile

You can also calculate a specific percentile of given input values using the
`percentile` command:

```console
$ echo 'import random\nfor i in range(1000):\n print(random.randint(0,10000))' \
  | python3 \
  | sta percentile 95
p95: 9436.00
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


[actions]: https://github.com/kongo2002/sta/actions/
[releases]: https://github.com/kongo2002/sta/releases/
