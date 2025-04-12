# __refitui__ : Review file tui 

Refitui consists in a text viewer that allows the user to perform the following actions:

1. Users can provide a filename as a command-line argument to display. This should be a valid filename that already exists. 
If the file does not exist, the program will display a message error and exit. 

    ```$> refitui /path/to/file```

2. The text viewer will load the file contents and display them on the terminal. If the number of lines in a file is more than the terminal height, 
the program will allow the user to scroll through the document, and repaint the next set of lines.

3. Users can use the up, down, left, and right keys to scroll throught the terminal.

4. Users can press __CTRL+Q__ to exit the text viewer.

5. User can edit the file typing with the keyboard. 

## TO-DO LIst
- [x] Basic setup and functionality of the command line tool

- [ ] Change header 

- [x] Add description for the command line tool with structopt

- [ ] Add functionality of writing

---

> This is my personal implementation of the Text viewer project presented on "Practical System Programming for Rust Developers" form Packt editorial.