Zoxide, but instead of being 2000 lines it's 200.

Nushell:

```shell
def --env __cd [...rest:string]  {
    let input = ($rest | str join)
    let output = if ($input | str starts-with '~') {
        # Turn '~' into C:/Users/<name>
        (cdeez ($input | path expand))
    } else {
        (cdeez $input)
    }

    # TODO: Swap to exit codes when they work in Nushell.
    if ($output | str starts-with 'cdeez') {
        echo $output # An error occured.
    } else {
        cd $output
    }
}

alias cd = __cd
```