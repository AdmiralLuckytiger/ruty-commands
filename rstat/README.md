# Rust source statistics 

A basic shell command line called rstat, short for "Rust source statistics". It accepts a directory as an argument.
It will generate a file count of Rust sources files, and source code metrics such as the number of blanks, comments, and actual lines of code within the directory structure. Futhermore, you could also analyse the binary files within the specified folder.

## Method of use:

For analise the sources of the current folder: `$> rstat src .` 

For analise the binaries of the current folder: `$> rstat bin .`

For print help: `$> rstat --help`
