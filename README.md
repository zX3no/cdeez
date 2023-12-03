Zoxide, but instead of being 2000 lines it's 200.

Add this to your terminal config.

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

Powershell: 

```powershell
function global:__cd {
    $input = $args -join ''
    $output = if ($input -match '^~') {
        # Turn '~' into C:/Users/<name>
        $output = Resolve-Path -ErrorAction SilentlyContinue -ErrorVariable error $input
        if ($error) {
            Write-Error "Cannot find path '$input'"
            return;
        }
        $output
    } else {
        $input
    }

    $result = cdeez $output
    if ($result.StartsWith('cdeez')) {
        Write-Output $result  # An error occurred.
    } else {
        Set-Location $result.Substring(4)
    }
}

Set-Alias -Name cd -Value __cd -Option AllScope -Scope Global -Force
```