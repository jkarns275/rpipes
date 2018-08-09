# Requirements
`rpipes` requires the ncursesw (wide-character supporting) development libraries to be installed.

On Ubuntu this library can be installed using `sudo apt-get install libncursesw5-dev`

# Usage

```
    rpipes 1.0
    Joshua Karns
    Prints moving pipes in the terminal
    
    USAGE:
        rpipes [OPTIONS]
    
    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information
    
    OPTIONS:
        -c, --charset <charset>      Sets the character set to be used
        -s, --colorset <colorset>    Sets the color set to be used
        -d, --delay <delay>          the delay between updates (ms).
        -M, --max_len <max_len>      The maximum length a pipe will be before it turns.
        -m, --min_len <min_len>      The minimum length of a pipe before it turns.
        -n, --numpipes <numpipes>    The number of pipes to be drawn at the same time.
```

# Todo
- [ ] Test on popular linux distros
- [ ] Test on Windows
- [ ] Test on MacOSX
