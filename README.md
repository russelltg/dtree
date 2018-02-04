# dtree
Dialog tree parser and Executor

Dtree a file format, library, and command line utility for dialog trees.

## Abstract 

![Dialog Tree](https://upload.wikimedia.org/wikipedia/commons/thumb/3/31/Dialog_tree_example.svg/399px-Dialog_tree_example.svg.png)

Dialog trees are a common occurence in video games, and it felt like a fun project to create an easy file format spec for these. 

## Terminology

Section: A state that the dialog tree can be in. Designated by a box in the above picture.

Mapping: A text input required to go from one section to another

## Syntax

Defining a section:

```
[                      start                 ] Welcome to the game! What would you like to do?
^ Text in brackets     ^ The name of the       ^ The text to show along with it
  define what's being     Section to describe
  described
```

Defining a mapping:

```
[ start                (to the house)                  ->house                     ] Go to the house
  ^ The section         ^ In parenthesis                 ^ The section               ^ The description for the mapping
    that the mapping      is the text that the             to travel to if
    starts at             player has to input              the text in 
                          to get to the next section       parenthesis is inputted

```

## Example

For example, the following dtree file `test/example.dtree`:

```dtree
[start] Welcome to this dtree file
[start (gym)->gym] Go to the gym
[start (videogames)->games] Play videogames

[gym] You made it to the gym! What a loser lol. Game over.

[games] Which game would you like to play?
[games (cs)->cs] Counter Strike
[games (cod)->cod] Call of Duty

[cs] Lmao what a shitty game. You got rekt. Game over.

[cod] You have 12 year old noobs scream at you because of the size of your mother. Wonderful.
```

A possible playthrough of this could look like:

```
$ dtree test/example.dtree
Welcome to this dtree file
(videogames) Play videogames
(gym) Go to the gym
> videogames
Which game would you like to play?
(cod) Call of Duty
(cs) Counter Strike
> cod
You have 12 year old noobs scream at you because of the size of your mother. Wonderful.
```


