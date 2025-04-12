use std::fs;
use std::io::{stdin, stdout, Write};
use termion::{
    event::Key,
    input::TermRead,
    raw::IntoRawMode,
    color,
    style,
};

/// Data strcuture that stores the document parsed in lines
struct Doc {
    lines: Vec<String>
}

/// Data structure that stores the curso position and to record the current size of the terminal
#[derive(Debug)] 
struct Coordinates {
    pub x: usize,
    pub y: usize,
}

/// Main data structure representing the text viewer
pub struct TextViewer {
    doc: Doc,
    doc_length: usize,
    cur_pos: Coordinates,
    terminal_size: Coordinates,
    file_name: String,
} 

impl TextViewer {
    /// Instantiate TextViewer and initializate
    pub fn init(file: &str) -> Self {

        // Initialize the buffer that is used to store the file contents
        let mut doc_file = Doc {
            lines: vec![]
        };

        // Read the file contents as a string
        let file_handle = fs::read_to_string(file).unwrap();

        // Read each line from the file and store it in ht Doc buffer
        for doc_line in file_handle.lines() {
            doc_file.lines.push(doc_line.to_string());
        }

        // Initailize the doc_length variable with the number of lines of the file
        let doc_length = file_handle.lines().count();

        // Use the termion crate to get the terminal size
        let size = termion::terminal_size().unwrap();

        // Create a new struct of the TextViewer type and return it from the init() method
        Self {
            doc: doc_file,
            cur_pos: Coordinates {
                x: 1,
                y: doc_length,
            },
            doc_length,
            terminal_size: Coordinates {
                x: size.0 as usize,
                y: size.1 as usize
            },
            file_name: file.into(),
        }
    }

    /// Displays the contents of the file on the terminal screen
    pub fn show_document(&mut self) {

        // 1. Store the current position of the cursor x and y coordinates in temp variables. 
        // This will be used to restore the cursor position in later step.
        let pos = &self.cur_pos;
        let (old_x, old_y) = (pos.x, pos.y);

        // 2. Using the Termination crate, clear the entire screen and move the cursor to row 1 and column 1 on the screen.
        println!("{}{}", termion::clear::All, termion::cursor::Goto(1,1));

        // 3. Print the header bar of the text viewer. A background color of black and foreground color of the white is used to print text.
        println!("{}{} Welcome to Super text viewer\r{}", color::Bg(color::White), color::Fg(color::Black), style::Reset);

        // 4. Display each line from the internal document buffer to the terminal screen.
        // Check whether the number of lines in the document is less than the terminal height.        
        if self.doc_length < self.terminal_size.y {
            // If so, display all lines from the input document on the terminal screen
            for line in 0..self.doc_length {
                println!("{}\r", self.doc.lines[line as usize]);
            }
        } else {
            // If the number of lines is greater than the terminal height, we have to display the document in parts.
            if pos.y <= self.terminal_size.y {
                for line in 0..self.terminal_size.y - 3 {
                    println!("{}\r", self.doc.lines[line as usize]);
                }
            } else {
                for line in pos.y - (self.terminal_size.y -3)..pos.y {
                    println!("{}\r", self.doc.lines[line as usize]);
                }
            }

        }

        // 5. Move the cursor to the bottom of the screen (using the terminal size y coordinate) to print the footer.
        println!("{}", termion::cursor::Goto(0, (self.terminal_size.y - 2) as u16));

        // 6. Print the footer text in red and with bold style. Print the number of lines in the document and filename to the footer.
        println!("{}{} line-count={} Filename: {}{}", color::Fg(color::Red), style::Bold, self.doc_length, self.file_name, style::Reset);

        // 7. Reset the cursor to the original position (which was saved to the temporary variable in step 1)
        self.set_pos(old_x, old_y)
    }

    /// Waits for user inputs to the process.
    /// If the user presses Ctrl + Q, the program exits. 
    pub fn run(&mut self) {
        // TODO: Handle posible error case.
        // stdout is used for display text to the terminal
        let mut stdout = stdout().into_raw_mode().unwrap();
        let stdin = stdin();
        
        // stdin.keys method is used for listen for the user inputs in a loop
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Ctrl('q') => {
                    // Exit the aplication
                    break;
                }, 
                Key::Left => {
                    // Move a cell to the left
                    self.dec_x();
                    self.show_document();
                }
                Key::Right => {
                    // Move a cell to the right
                    self.inc_x();
                    self.show_document();
                }
                Key::Up => {
                    // Move a cell up
                    self.dec_y();
                    self.show_document();
                }
                Key::Down => {
                    // Move a cell down
                    self.inc_y();
                    self.show_document();
                }
                Key::Backspace => {
                    // ¿?
                    self.dec_x();
                }
                _ => {}
            }
            stdout.flush().unwrap();
        }
    }

    /// Helper method that synchronizes the internal cursor tracking field (the cur_pos field of the TextViewer strcut)
    ///  and the on-screen cursor position
    fn set_pos(&mut self, x: usize, y: usize) {
        self.cur_pos.x = x;
        self.cur_pos.y = y;

        println!("{}", termion::cursor::Goto(self.cur_pos.x as u16, self.cur_pos.y as u16));
    }

    /// Helper method decrement the coordinate x and repositionate the cursor on the screen 
    fn dec_x(&mut self) {
        if self.cur_pos.x > 1 {
            self.cur_pos.x -= 1;
        }

        println!("{}", termion::cursor::Goto(self.cur_pos.x as u16, self.cur_pos.y as u16));
    }

    /// Helper method decrement the coordinate y and repositionate the cursor on the screen 
    fn dec_y(&mut self) {
        if self.cur_pos.y > 1 {
            self.cur_pos.y -= 1;
        }

        println!("{}", termion::cursor::Goto(self.cur_pos.x as u16, self.cur_pos.y as u16));
    }

    /// Helper method increment the coordinate x and repositionate the cursor on the screen 
    fn inc_x(&mut self) {
        if self.cur_pos.x < self.terminal_size.x {
            self.cur_pos.x += 1;
        }

        println!("{}", termion::cursor::Goto(self.cur_pos.x as u16, self.cur_pos.y as u16));        
    }

    /// Helper method increment the coordinate y and repositionate the cursor on the screen 
    fn inc_y(&mut self) {
        if self.cur_pos.y < self.doc_length {
            self.cur_pos.y += 1;
        }

        println!("{}", termion::cursor::Goto(self.cur_pos.x as u16, self.cur_pos.y as u16));
    }

}