use mcp23017;
use mcp23017::MCP23017;
use rppal::i2c;

// Constants for LCD commands
const LCD_CLEARDISPLAY: u8 = 0x01;
const LCD_RETURNHOME: u8 = 0x02;
const LCD_ENTRYMODESET: u8 = 0x04;
const LCD_DISPLAYCONTROL: u8 = 0x08;
const LCD_CURSORSHIFT: u8 = 0x10;
const LCD_FUNCTIONSET: u8 = 0x20;
const LCD_SETCGRAMADDR: u8 = 0x40;
const LCD_SETDDRAMADDR: u8 = 0x80;

// Entry flags
const LCD_ENTRYLEFT: u8 = 0x02;
const LCD_ENTRYSHIFTDECREMENT: u8 = 0x00;

// Control flags
const LCD_DISPLAYON: u8 = 0x04;
const LCD_CURSORON: u8 = 0x02;
const LCD_CURSOROFF: u8 = 0x00;
const LCD_BLINKON: u8 = 0x01;
const LCD_BLINKOFF: u8 = 0x00;

// Move flags
const LCD_DISPLAYMOVE: u8 = 0x08;
const LCD_MOVERIGHT: u8 = 0x04;
const LCD_MOVELEFT: u8 = 0x00;

// Function set flags
const LCD_4BITMODE: u8 = 0x00;
const LCD_2LINE: u8 = 0x08;
const LCD_1LINE: u8 = 0x00;
const LCD_5X8DOTS: u8 = 0x00;

// Row offsets for up to 4 rows
const LCD_ROW_OFFSETS: [u8; 4] = [0x00, 0x40, 0x14, 0x54];

// The MCP23017 pins attached to each LCD pin, all are outputs
// i.e. the LCD's reset pin is on MCP23017 pin 15
enum LcdPins {
    #[allow(non_camel_case_types)]
    RESET,
    #[allow(non_camel_case_types)]
    ENABLE,
    #[allow(non_camel_case_types)]
    D4,
    #[allow(non_camel_case_types)]
    D5,
    #[allow(non_camel_case_types)]
    D6,
    #[allow(non_camel_case_types)]
    D7,
    #[allow(non_camel_case_types)]
    RED,
    #[allow(non_camel_case_types)]
    GREEN,
    #[allow(non_camel_case_types)]
    BLUE,
    #[allow(non_camel_case_types)]
    RW,
}

impl LcdPins {
    pub const fn mcp_pin(&self) -> u8 {
        match *self {
            Self::RESET => 15,
            Self::ENABLE => 13,
            Self::D4 => 12,
            Self::D5 => 11,
            Self::D6 => 10,
            Self::D7 => 9,
            Self::RED => 6,
            Self::GREEN => 7,
            Self::BLUE => 8,
            Self::RW => 14,
        }
    }
    pub const PINS: [u8; 10] = [
        Self::mcp_pin(&Self::RESET),
        Self::mcp_pin(&Self::ENABLE),
        Self::mcp_pin(&Self::D4),
        Self::mcp_pin(&Self::D5),
        Self::mcp_pin(&Self::D6),
        Self::mcp_pin(&Self::D7),
        Self::mcp_pin(&Self::RED),
        Self::mcp_pin(&Self::GREEN),
        Self::mcp_pin(&Self::BLUE),
        Self::mcp_pin(&Self::RW),
    ];
}

// MCP23017 pins attached to buttons on the board All are inputs
enum BoardPins {
    #[allow(non_camel_case_types)]
    LEFT_BUTTON,
    #[allow(non_camel_case_types)]
    RIGHT_BUTTON,
    #[allow(non_camel_case_types)]
    UP_BUTTON,
    #[allow(non_camel_case_types)]
    DOWN_BUTTON,
    #[allow(non_camel_case_types)]
    SELECT_BUTTON,
}

impl BoardPins {
    const fn mcp_pin(&self) -> u8 {
        match *self {
            Self::LEFT_BUTTON => 4,
            Self::RIGHT_BUTTON => 1,
            Self::UP_BUTTON => 3,
            Self::DOWN_BUTTON => 2,
            Self::SELECT_BUTTON => 0,
        }
    }
    const PINS: [u8; 5] = [
        Self::mcp_pin(&Self::LEFT_BUTTON),
        Self::mcp_pin(&Self::RIGHT_BUTTON),
        Self::mcp_pin(&Self::UP_BUTTON),
        Self::mcp_pin(&Self::DOWN_BUTTON),
        Self::mcp_pin(&Self::SELECT_BUTTON),
    ];
}

#[derive(Clone, Copy, PartialEq)]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
}

pub struct CharacterLcdRgb {
    mcp: MCP23017<rppal::i2c::I2c>,
    text: String,
    columns: u8,
    lines: u8,
    display_control: u8,
    display_function: u8,
    display_mode: u8,
    row: u8,
    column: u8,
    column_align: bool,
    text_direction: TextDirection,
    colour: [u8; 3], // RGB backlight colour
}

type BoardError = mcp23017::Error<rppal::i2c::Error>;

impl CharacterLcdRgb {
    pub fn new(
        i2c: rppal::i2c::I2c,
        columns: u8,
        lines: u8,
        address: Option<u8>,
    ) -> Result<Self, BoardError> {
        // Initialise the GPIO expander chip
        let mut mcp = match address {
            Some(addr) => MCP23017::new(i2c, addr)?,
            None => MCP23017::new(i2c, 0x20)?, // Default I2C address
        };

        // Initialise GPIO expander's pin directions etc.
        Self::setup_pins(&mut mcp)?;

        // Create initial state
        let mut lcd = Self {
            mcp,
            text: String::new(),
            columns,
            lines,
            display_control: LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF,
            display_function: LCD_4BITMODE | LCD_1LINE | LCD_2LINE | LCD_5X8DOTS,
            display_mode: LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT,
            row: 0,
            column: 0,
            column_align: false,
            text_direction: TextDirection::LeftToRight,
            colour: [128, 128, 128],
        };

        lcd.initialise()?;
        Ok(lcd)
    }

    fn setup_pins(mcp: &mut MCP23017<rppal::i2c::I2c>) -> Result<(), BoardError> {
        for mcp_pin in LcdPins::PINS.iter() {
            mcp.pin_mode(*mcp_pin, mcp23017::PinMode::OUTPUT)?;
        }
        for mcp_pin in BoardPins::PINS.iter() {
            mcp.pin_mode(*mcp_pin, mcp23017::PinMode::INPUT)?;
            mcp.pull_up(*mcp_pin, true)?;
        }

        Ok(())
    }

    fn initialise(&mut self) -> Result<(), BoardError> {
        // Initialise into 4-bit mode
        self.write_ctrl(0x33)?;
        self.write_ctrl(0x32)?;

        // Initialise display control
        self.write_ctrl(LCD_DISPLAYCONTROL | self.display_control)?;

        // Initialise display function
        self.write_ctrl(LCD_FUNCTIONSET | self.display_function)?;

        // Initialise display mode
        self.write_ctrl(LCD_ENTRYMODESET | self.display_mode)?;

        self.clear()?;

        Ok(())
    }

    /// Clears the contents of the display
    pub fn clear(&mut self) -> Result<(), BoardError> {
        self.write_ctrl(LCD_CLEARDISPLAY)?;
        Self::delay_ms(3); // Datasheet specified
        Ok(())
    }

    /// Sets the cursor to the home position
    pub fn cursor_home(&mut self) -> Result<(), BoardError> {
        self.write_ctrl(LCD_RETURNHOME)?;
        CharacterLcdRgb::delay_ms(3);
        Ok(())
    }

    /// Sets the cursor to an arbitrary position
    pub fn set_cursor_position(&mut self, column: u8, row: u8) -> Result<(), BoardError> {
        let row = row.min(self.lines - 1);
        let column = column.min(self.columns - 1);
        self.write_ctrl(LCD_SETDDRAMADDR | (column + LCD_ROW_OFFSETS[row as usize]))?;
        self.row = row;
        self.column = column;
        Ok(())
    }

    /// Activates or deactivates the display
    pub fn set_activity(&mut self, active: bool) -> Result<(), BoardError> {
        if active {
            self.display_control = self.display_control | LCD_DISPLAYON;
        } else {
            self.display_control = self.display_control & !LCD_DISPLAYON;
        }
        self.write_ctrl(LCD_DISPLAYCONTROL | self.display_control)?;
        Ok(())
    }

    /// Whether or not the display is currently active
    pub fn get_activity(&self) -> bool {
        self.display_control & LCD_DISPLAYON == LCD_DISPLAYON
    }

    /// Whether or not the display's cursor is currently blinking
    pub fn get_blink(&self) -> bool {
        self.display_control & LCD_BLINKON == LCD_BLINKON
    }

    /// Activate or deactivate cursor blinking
    pub fn set_blink(&mut self, blink: bool) -> Result<(), BoardError> {
        if blink {
            self.display_control |= LCD_BLINKON;
        } else {
            self.display_control &= !LCD_BLINKON;
        }
        self.write_ctrl(LCD_DISPLAYCONTROL | self.display_control)?;
        Ok(())
    }

    /// Make the cursor visible or invisible
    pub fn set_cursor_visible(&mut self, show: bool) -> Result<(), BoardError> {
        if show {
            self.display_control |= LCD_CURSORON;
        } else {
            self.display_control &= !LCD_CURSORON;
        }
        self.write_ctrl(LCD_DISPLAYCONTROL | self.display_control)?;
        Ok(())
    }

    /// Returns whether the cursor is currently visible
    pub fn get_cursor_visible(&self) -> bool {
        self.display_control & LCD_CURSORON == LCD_CURSORON
    }

    /// Moves the contents of the display left one column
    fn move_left(&mut self) -> Result<(), BoardError> {
        self.write_ctrl(LCD_CURSORSHIFT | LCD_DISPLAYMOVE | LCD_MOVELEFT)?;
        Ok(())
    }

    /// Moves the contents of the display right one column
    fn move_right(&mut self) -> Result<(), BoardError> {
        self.write_ctrl(LCD_CURSORSHIFT | LCD_DISPLAYMOVE | LCD_MOVERIGHT)?;
        Ok(())
    }

    /// Sets the direction of the text displayed
    pub fn set_direction(&mut self, direction: TextDirection) -> Result<(), BoardError> {
        self.text_direction = direction;
        match self.text_direction {
            TextDirection::LeftToRight => {
                self.display_mode |= LCD_ENTRYLEFT;
                self.write_ctrl(LCD_ENTRYMODESET | self.display_mode)?;
                Ok(())
            }
            TextDirection::RightToLeft => {
                self.display_mode &= !LCD_ENTRYLEFT;
                self.write_ctrl(LCD_ENTRYMODESET | self.display_mode)?;
                Ok(())
            }
        }
    }

    /// Gets the direction of the displayed text
    pub fn get_direction(&self) -> TextDirection {
        self.text_direction
    }

    /// Sets the text message displayed
    pub fn set_text(&mut self, text: String) -> Result<(), BoardError> {
        let chars: Vec<char> = text.chars().collect();
        self.text = text;
        let mut line = self.row;

        // Set the initial cursor position
        let mut col = if self.text_direction == TextDirection::LeftToRight {
            self.column
        } else {
            self.columns - 1 - self.column
        };
        self.set_cursor_position(col, line)?;

        for &c in chars.iter() {
            match c {
                '\n' => {
                    line = (line + 1).min(self.lines - 1);
                    col = match (self.text_direction, self.column_align) {
                        (TextDirection::LeftToRight, true) => self.column,
                        (TextDirection::LeftToRight, false) => 0,
                        (TextDirection::RightToLeft, true) => self.column,
                        (TextDirection::RightToLeft, false) => self.columns - 1,
                    };
                    self.set_cursor_position(col, line)?;
                }
                c => {
                    self.write_char(c as u8)?;
                }
            };
        }

        self.column = 0;
        self.row = 0;
        Ok(())
    }

    /// Writes an arbitrary byte to the display
    fn write_byte(&mut self, value: u8) -> Result<(), BoardError> {
        // Write four upper bits
        self.mcp
            .digital_write(LcdPins::D4.mcp_pin(), (value >> 4) & 1 == 1)?;
        self.mcp
            .digital_write(LcdPins::D5.mcp_pin(), (value >> 5) & 1 == 1)?;
        self.mcp
            .digital_write(LcdPins::D6.mcp_pin(), (value >> 6) & 1 == 1)?;
        self.mcp
            .digital_write(LcdPins::D7.mcp_pin(), (value >> 7) & 1 == 1)?;

        self.pulse_enable()?;

        // Write four lower bits
        self.mcp
            .digital_write(LcdPins::D4.mcp_pin(), (value >> 0) & 1 == 1)?;
        self.mcp
            .digital_write(LcdPins::D5.mcp_pin(), (value >> 1) & 1 == 1)?;
        self.mcp
            .digital_write(LcdPins::D6.mcp_pin(), (value >> 2) & 1 == 1)?;
        self.mcp
            .digital_write(LcdPins::D7.mcp_pin(), (value >> 3) & 1 == 1)?;

        self.pulse_enable()?;

        Ok(())
    }

    /// Writes an 8-bit `value` to the display in char-mode, only for character bits
    fn write_char(&mut self, value: u8) -> Result<(), BoardError> {
        // Set RS pin based on char mode
        self.mcp.digital_write(LcdPins::RESET.mcp_pin(), true)?;
        self.write_byte(value)
    }

    /// Writes an 8-bit `value` ot the display in data-mode, used for writing display commands
    fn write_ctrl(&mut self, value: u8) -> Result<(), BoardError> {
        // Don't set RS pin based on char mode
        self.mcp.digital_write(LcdPins::RESET.mcp_pin(), false)?;
        self.write_byte(value)
    }

    pub fn read_button(&mut self, button: BoardPins) -> Result<bool, BoardError> {
        Ok(self.mcp.digital_read(button.mcp_pin())?)
    }

    fn pulse_enable(&mut self) -> Result<(), BoardError> {
        self.mcp.digital_write(LcdPins::ENABLE.mcp_pin(), false)?;
        Self::delay_us(1);
        self.mcp.digital_write(LcdPins::ENABLE.mcp_pin(), true)?;
        Self::delay_us(1);
        self.mcp.digital_write(LcdPins::ENABLE.mcp_pin(), false)?;
        Self::delay_us(1);
        Ok(())
    }

    fn delay_ms(ms: u64) {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }

    fn delay_us(us: u64) {
        std::thread::sleep(std::time::Duration::from_micros(us));
    }
}
