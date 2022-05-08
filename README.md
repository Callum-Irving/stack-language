# Stack Oriented Programming Lanaguage
An unnamed stack-oriented programming language.

If you don't know what a stack-oriented programming language is, see [Forth](https://www.forth.com/forth/).

## How to use the language

The language doesn't really have a type system. Everything is 64 bit integers.


### Hello, world!

A program consists of a list of constants and functions. An example of a program the prints "Hello, world!" is:

```
const msg = "Hello, world!\n"

let main () = {
  msg puts
}
```

### Builtin functions

The list of builtin functions is:

- `dup (a -> a a)` duplicates the last item on the stack
- `drop (a -> )` deletes the last item on the stack
- `puts (a -> )` prints a string
- `print (a -> )` prints a number
- `swap (a b -> b a)` swaps the last two items on the stack
- `read (ptr -> )` reads from stdin into a buffer

### Declaring constants

Constants can either be just an integer or an array of bytes which are represented by a pointer. The syntax for declaring a constant is as follows:

```
const <name> = (<int> OR <int> bytes)
```

### Declaring functions

Functions are declared using `let <name> (<type list> -> OPTIONAL <type list>) = { <body> }`

There is no type checking done on the parameters and return values yet, but they must be either `int` or `ptr` for the parser to accept it.

# Inspired by

[porth](https://gitlab.com/tsoding/porth)
