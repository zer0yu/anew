<h1 align="center">anew</h1>
<h3 align="center">A tool for adding new lines to files written in Rust.</h3>

The tool aids in appending lines from stdin to a file, but only if they don't already appear in the file.
Outputs new lines to `stdout` too, making it a bit like a `tee -a` that removes duplicates.

## Install

Via cargo install
```
cargo install anew
```

Manual installation
```
git clone https://github.com/zer0yu/anew
cd anew
cargo build
```
or you can download the binary from [releases](https://github.com/zer0yu/anew/releases)

## Usage

```
❯ anew --help
A tool for adding new lines to files, skipping duplicates

Usage: anew [OPTIONS] <FILEPATH>

Arguments:
  <FILEPATH>  Destination file

Options:
  -q, --quiet-mode  Do not output new lines to stdout
  -s, --sort        Sort lines (natsort)
  -t, --trim        Trim whitespaces
  -r, --rewrite     Rewrite existing destination file to remove duplicates
  -d, --dry-run     Do not write to file, only output what would be written
  -h, --help        Print help
  -V, --version     Print version
```

## Usage Example

Here, a file called `things.txt` contains a list of numbers. `newthings.txt` contains a second
list of numbers, some of which appear in `things.txt` and some of which do not. `anew` is used
to append the latter to `things.txt`.

Usage 1: Add differences to `things.txt`
```
❯ cat things.txt
One
Zero
Two
One

❯ cat newthings.txt
Three
One
Five
Two
Four

❯ cat newthings.txt | anew things.txt
Three
Five
Four

❯ cat things.txt
One
Zero
Two
One
Three
Five
Four
```

Usage 2: Disable terminal output
```
❯ cat newthings.txt | anew things.txt -q
Three
Five
Four
``` 

Usage 3: Sorting the contents of `things.txt` after adding new differences
```
❯ cat newthings.txt | ./anew things.txt -q -s

❯ cat things.txt
Five
Four
One
Three
Two
Zero
```
PS: 
1. anew uses the fastest sorting algorithm, so don't worry about efficiency.
2. Sort mode automatically de-duplicates, so `-s` and `-r` do not need to be used at the same time.

Usage 4: De-duplication of `things.txt` after adding new differences
```
❯ cat newthings.txt | ./anew things.txt -q -r

❯ cat things.txt
One
Zero
Two
Three
Five
Four
```

## Efficiency Comparison

We use two files `newoutput.txt` and `output.txt` of size 10MB as input to the program to compare the difference in speed between tomnomnom's Go implementation, rwese's Rust implementation, and this project's Rust implementation.

This project
```
❯ time cat newoutput.txt | ./anew output.txt -q
cat newoutput.txt  0.00s user 0.02s system 1% cpu 1.398 total
./anew output.txt -q  1.46s user 0.22s system 97% cpu 1.717 total
```

anew implemented by rwese
```
❯ time cat newoutput.txt | ./anew_rwese output.txt -q
cat newoutput.txt  0.00s user 0.02s system 0% cpu 2:28.38 total
./anew output.txt -q  6.95s user 101.08s system 72% cpu 2:29.49 total
```

anew implemented by tomnomnom
```
❯ time cat newoutput.txt | ./anew_go -q output.txt
cat newoutput.txt  0.00s user 0.02s system 0% cpu 4.797 total
./anew_go -q output.txt  2.11s user 3.14s system 108% cpu 4.838 total
```

As can be seen from the above, the project has been implemented most efficiently!

## References

1. [anew@tomnomnom](https://github.com/tomnomnom/anew)
2. [anew@rwese](https://github.com/rwese/anew)