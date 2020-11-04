use std::{thread, time, process};
use rand::Rng;
use pixels::SurfaceTexture;
use pixels::Pixels;
use winit::event_loop::EventLoop;
use winit::dpi::{PhysicalSize, LogicalSize, LogicalPosition};
use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

// Create Tetromino pieces
const TETROMINO: [&str; 7] = [
    "..X...X...X...X.",
    "..X..XX...X.....",
    ".....XX..XX.....",
    ".....XX..XX.....",
    ".X...XX...X.....",
    ".X...X...XX.....",
    "..X...X..XX....."];


// Set field dimensions
const NFIELDWIDTH: u32 = 12;
const NFIELDHEIGHT: u32 = 18;

// Set screen dimensions
const NSCREENWIDTH: u32 = NFIELDWIDTH;
const NSCREENHEIGHT: u32 = NFIELDHEIGHT;


fn rotate(px: i32, py: i32, r: i32) -> Result<i32, &'static str> {
    let rotation = r % 4;
    match rotation {
        0 => return Ok(py * 4 + px),
        1 => return Ok(12 + py - (px * 4)),
        2 => return Ok(15 - (py * 4) - px),
        3 => return Ok(3 - py + (px * 4)),
        _ => Err("invalid rotation"),
    }
}

fn does_piece_fit(p_field: &[char], n_tetromino: i32, n_rotation: i32, n_pos_x: i32, n_pos_y: i32) -> bool {
    for px in 0..4 {
        for py in 0..4 {
            // get index into piece
            let pi: i32 = rotate(px, py, n_rotation).unwrap();
            // get index into field
            let fi: i32 = (n_pos_y + py) * NFIELDWIDTH as i32 + (n_pos_x + px);

            // bounds check
            if n_pos_x + px >= 0 && n_pos_x + px < NFIELDWIDTH as i32 {
                if n_pos_y + py >= 0 && n_pos_y + py < NFIELDHEIGHT as i32 {
                    if TETROMINO[n_tetromino as usize].as_bytes()[pi as usize] != '.' as u8 && p_field[fi as usize] != '0' {
                        return false;
                    }
                }
            }
        }
    }
    true
}

fn create_window(title: &str, event_loop: &EventLoop<()>) -> (winit::window::Window, u32, u32, f64) {
    let window = winit::window::WindowBuilder::new()
        .with_visible(false)
        .with_title(title)
        .build(&event_loop)
        .unwrap();
    let hidpi_factor = window.scale_factor();

    let width = NSCREENWIDTH as f64;
    let height = NSCREENHEIGHT as f64;
    let (monitor_width, monitor_height) = {
        let size = window.current_monitor().unwrap().size();
        (
            size.width as f64 / hidpi_factor,
            size.height as f64 / hidpi_factor,
        )
    };
    let scale = (monitor_height / height * 2.0 / 3.0).round();
    let min_size = PhysicalSize::new(width, height).to_logical::<f64>(hidpi_factor);
    let default_size = LogicalSize::new(width * scale, height * scale);
    let center = LogicalPosition::new(
        (monitor_width - width * scale) / 2.0,
        (monitor_height - height * scale) / 2.0,
    );
    window.set_inner_size(default_size);
    window.set_min_inner_size(Some(min_size));
    window.set_outer_position(center);
    window.set_visible(true);

    let size = default_size.to_physical::<f64>(hidpi_factor);

    (
        window,
        size.width.round() as u32,
        size.height.round() as u32,
        hidpi_factor,
    )
}


fn main() {
    // Winit setup
    let event_loop = EventLoop::new();
    let (window, width, height, mut _hidpi_factor) = create_window("Tetris", &event_loop);
    let mut input = WinitInputHelper::new();

    // Pixels setup
    let surface_texture = SurfaceTexture::new(width, height, &window);
    let mut pixels_buffer = Pixels::new(NSCREENWIDTH, NSCREENHEIGHT, surface_texture).unwrap();

    // Display buffer setup
    let mut p_field: [char; (NFIELDWIDTH * NFIELDHEIGHT) as usize] = array_init::array_init(|i: usize| {
        let i = i as u32;
        return if (i + 1) % 12 <= 1 || i > NFIELDWIDTH * NFIELDHEIGHT - NFIELDWIDTH {
            '9'
        } else {
            '0'
        };
    });
    let mut screen: [char; (NFIELDWIDTH * NFIELDHEIGHT) as usize] = ['0'; (NFIELDWIDTH * NFIELDHEIGHT) as usize];

    let mut rng = rand::thread_rng();
    let mut n_current_piece: i32 = 1;
    let mut n_current_rotation: i32 = 0;
    let mut n_current_x: i32 = NFIELDWIDTH as i32 / 2;
    let mut n_current_y: i32 = 0;
    let mut n_speed: i32 = 20;
    let mut n_speed_count: i32 = 0;
    let mut n_piece_count: i32 = 0;
    let mut n_score: i32 = 0;
    let mut v_lines = Vec::new();
    let mut b_game_over: bool = false;


    // === GAME LOOP ===============================================================================
    event_loop.run(move |event, _, _control_flow| {
        if b_game_over {
            println!("GAME OVER!");
            println!("SCORE: {}!", n_score);
            process::exit(0);
        }
        // Game timing
        thread::sleep(time::Duration::from_millis(50));
        n_speed_count += 1;

        // Input detection
        if input.update(&event) {

        let input_right = input.key_pressed(VirtualKeyCode::Right);
        let input_left = input.key_pressed(VirtualKeyCode::Left);
        let input_down = input.key_pressed(VirtualKeyCode::Down);
        let input_rotate = input.key_pressed(VirtualKeyCode::Up);

        // Handle player movement
        n_current_x += if input_right && does_piece_fit(&p_field, n_current_piece, n_current_rotation, n_current_x + 1, n_current_y) { 1 } else { 0 };
        n_current_x -= if input_left && does_piece_fit(&p_field, n_current_piece, n_current_rotation, n_current_x - 1, n_current_y) { 1 } else { 0 };
        n_current_y += if input_down && does_piece_fit(&p_field, n_current_piece, n_current_rotation, n_current_x, n_current_y + 1) { 1 } else { 0 };
        n_current_rotation += if input_rotate && does_piece_fit(&p_field, n_current_piece, n_current_rotation + 1, n_current_x, n_current_y) { 1 } else { 0 };

        }

        // Force piece down on interval
        if n_speed_count == n_speed {
            // Update difficulty every 50 pieces
            n_speed_count = 0;
            n_piece_count += 1;
            if n_piece_count % 50 == 0 {
                if n_speed >= 10 {
                    n_speed -= 1;
                }
            }

            // Test if piece can be moved down
            if does_piece_fit(&p_field, n_current_piece, n_current_rotation, n_current_x, n_current_y + 1) {
                n_current_y += 1;
            } else {
                // Lock the piece since it cannot be moved during forced move
                for px in 0..4 {
                    for py in 0..4 {
                        if TETROMINO[n_current_piece as usize].as_bytes()[rotate(px, py, n_current_rotation).unwrap() as usize] != '.' as u8 {
                            // TODO instead of 7,8,9, use different colors for different pieces (both static and moving)
                            p_field[((n_current_y + py) * NFIELDWIDTH as i32 + (n_current_x + px)) as usize] = '7';
                        }
                    }
                }

                // Check for lines
                for py in 0..4 {
                    if n_current_y + py < NFIELDHEIGHT as i32 - 1 {
                        let mut b_line: bool = true;
                        for px in 1..NFIELDWIDTH as i32 - 1 {
                            b_line &= (p_field[((n_current_y + py) * NFIELDWIDTH as i32 + px) as usize]) != '0';
                        }
                        if b_line {
                            // Remove line, set to 0
                            for px in 1..NFIELDWIDTH as i32 - 1 {
                                p_field[((n_current_y + py) * NFIELDWIDTH as i32 + px) as usize] = '0';
                            }
                            v_lines.push(n_current_y + py);
                        }
                    }
                }

                n_score += 25;
                if !v_lines.is_empty() {
                    n_score += (1 << v_lines.len()) * 100;
                }
                // Pick New Piece and reset location to top
                n_current_x = NFIELDWIDTH as i32 / 2;
                n_current_y = 0;
                n_current_rotation = 0;

                n_current_piece = rng.gen_range(0, 7);

                // If piece does not fit at top, game over
                b_game_over = !does_piece_fit(&p_field, n_current_piece, n_current_rotation, n_current_x, n_current_y);
            }
        }

        // Copy field to screen buffer
        for pos in 0..NFIELDWIDTH * NFIELDHEIGHT {
            screen[pos as usize] = p_field[pos as usize]
        }

        // Draw Current Piece on screen buffer
        for px in 0..4 {
            for py in 0..4 {
                if TETROMINO[n_current_piece as usize].as_bytes()[rotate(px, py, n_current_rotation).unwrap() as usize] != '.' as u8 {
                    screen[((n_current_y + py) * NFIELDWIDTH as i32 + (n_current_x + px)) as usize] = '8';
                }
            }
        }

        // Set pixel color
        let frame = pixels_buffer.get_frame();
        let mut i: usize = 0;
        for pixel in frame.chunks_exact_mut(4) {
            pixel[0] = 0x00; // R
            pixel[1] = 0x00; // G
            pixel[2] = 0x00; // B
            pixel[3] = 0xff; // A
        }
        for pixel in frame.chunks_exact_mut(4) {
            if screen[i] == '0' {
                pixel[0] = 0x00; // R
                pixel[1] = 0x00; // G
                pixel[2] = 0x00; // B
            } else if screen[i] == '7' {
                pixel[0] = 0xFF; // R
                pixel[1] = 0x00; // G
                pixel[2] = 0x00; // B
            } else if screen[i] == '8' {
                pixel[0] = 0x00; // R
                pixel[1] = 0xFF; // G
                pixel[2] = 0x00; // B
            } else if screen[i] == '9' {
                pixel[0] = 0x00; // R
                pixel[1] = 0x00; // G
                pixel[2] = 0xFF; // B
            }

            pixel[3] = 0xff; // A
            i += 1;
        }
        // Animate/move line down
        if !v_lines.is_empty() {
            match pixels_buffer.render() {
                Ok(..) => (),
                Err(err) => println!("Error: {:?}", err),
            }
            thread::sleep(time::Duration::from_millis(400));

            for v in &v_lines {
                for px in 1..NFIELDWIDTH as i32 - 1 {
                    for py in (1..*v + 1).rev() {
                        p_field[(py as i32 * NFIELDWIDTH as i32 + px) as usize] = p_field[((py - 1) as i32 * NFIELDWIDTH as i32+ px) as usize];
                    }
                    p_field[px as usize] = '0';
                }
            }

            v_lines.clear();
        }

        // Render window
        match pixels_buffer.render() {
            Ok(..) => (),
            Err(err) => println!("Error: {:?}", err),
        }
    });
}
