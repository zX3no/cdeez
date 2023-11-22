Zoxide, but instead of being 2000 lines it's 100.

The program outputs the desired path.

Nushell:

```shell
def --env __cd [...rest:string]  {
    let output = (cdeez $rest)
    # TODO: Swap to exit codes when they work.
    if ($output | str starts-with 'cdeez') {
        # An error occured.
        echo $output
    } else {
        cd $output
    }
}

alias cd = __cd
```