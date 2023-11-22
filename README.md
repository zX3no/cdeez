Zoxide, but instead of being 2000 lines it's 200.

Nushell:

```shell
def --env __cd [...rest:string]  {
    let output = (cdeez $rest)
    # TODO: Swap to exit codes when they work in Nushell.
    if ($output | str starts-with 'cdeez') {
        # An error occured.
        echo $output
    } else {
        cd $output
    }
}

alias cd = __cd
```