# Marrow

A program to make TTS Bloodless decks.

## Usage

Make a text file for your deck, such as

```
2x Ant Queen
5x LMR
5x Deranged Researcher
```

The name after the number must be the name of the image file, not the name of the card. For example, LMR is the name of the image for Lost Man's right-facing version.

Then, run `marrow -i ./path/to/your/deck/file -o ./output_name.json -t` to put the deck in your TTS saved objects folder as an object.

You can now run TTS and you'll find the new deck object named `output_name`.

For other options, use `marrow -h`
