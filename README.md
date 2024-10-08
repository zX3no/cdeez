> [!WARNING]  
> Windows and MacOS only.

Zoxide, but instead of being 2000 lines it's 200.

Add this to your terminal config.

Nushell:

```shell
def --env __cd [...rest:string]  {
    let result = (cdeez ($rest | str join " "))
    if ($result | str starts-with 'cdeez') {
        echo $result # An error occured.
    } else {
        cd $result
    }
}

alias cd = __cd
```

Powershell: 

```powershell
function global:__cd {
    $result = cdeez ($args -join ' ')
    if ($result.StartsWith('cdeez')) {
        Write-Output $result # An error occurred.
    } else {
        Set-Location $result.Substring(4)
    }
}

Set-Alias -Name cd -Value __cd -Option AllScope -Scope Global -Force
```

Zsh:

```bash
function __cd() {
    result=$(cdeez "$@")
    if [[ $result == cdeez* ]]; then
        echo "$result" # An error occurred.
    else
        builtin cd "${result}"
    fi
}

alias cd="__cd"
```