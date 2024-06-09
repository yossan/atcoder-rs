# atcoder-rs

## Installation

```
$ git clone https://github.com/yossan/atcoder-rs.git
$ cd atcoder-rs
$ cargo install --path .
```

## Usage

### `new`

Generate a project for AtCoder.
Example: Create the abc326 project

```
$ atcoder new abc326
```

### `testcase`

Run test cases.

Example: Run the test cases for `A.rs`
Place the following directories under `testcase/a`:

- `in` : List of input test cases
- `out` : Corresponding list of expected results for input test cases

To run all test cases under the in directory:

```
$ atcoder testcase A
```

To run only the 1.txt and 3.txt test cases from the in directory:

```
$ atcoder testcase A -i 1 2 3
```

If the source file name and the parent directory containing the test cases have different names:

```
$ atcoder testcase A2 -d A
```
