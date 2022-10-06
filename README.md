``` 
   _____ _______ _  ___      _____  
  / ____|__   __| |/ / |    |  __ \ 
 | (___    | |  | ' /| |    | |__) |
  \___ \   | |  |  < | |    |  _  / 
  ____) |  | |  | . \| |____| | \ \ 
 |_____/   |_|  |_|\_\______|_|  \_\
                                    
                                    
```
Pronounced 'stickler'.

# Summary:
My favourite feature of rust-doc (the tool that automatically generates documentation from a paticular kind of docstring you can use in rust codebases) is
the keyword linking.
My least favourite feature of rust-doc is trying to remember to do said linking, or syntax highlighting in general as, I usually edit markdown (what rust-doc essentially makes) in a specific tool for it.

So, I made a tool to do it for me -- if you too are like me, perhaps it'll not only bring you joy, but also up the quality of your crate's docs.

# Installation:
- `cargo install --git https://github.com/alphastrata/stklr `

## NOTE THIS APP IS BETA AT BEST, IT'S A WIP.

# Usage:
> Backup your codebase before using this, use source-control.
- `stklr` # will bring up the help menu.
- `stklr preview` #will show you changes it wants to make, changes are in green, source files and line numbers etc are all there.
- `stklr fix`# will make changes to all the files you saw above, with `preview`.

## Bugs:
- Make in issue/PR -- include the text that threw it off.

## Contributing:
- not yet.

## More release platforms etc:
- later, not in beta stage.




