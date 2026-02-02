use phasetida_core::TOUCH_STATES;

pub fn process_touch_info(input_buffer: &js_sys::Uint8Array){
    let mut cursor = 0;
    TOUCH_STATES.with_borrow_mut(|touch_buf| {
        let mut enable_addition = [false; 30];
        loop {
            let check_byte = input_buffer.get_index(cursor);
            cursor += 1;
            if check_byte == 0 {
                break;
            }
            let id = input_buffer.get_index(cursor);
            cursor += 1;
            let x_slice = [
                input_buffer.get_index(cursor),
                input_buffer.get_index(cursor + 1),
                input_buffer.get_index(cursor + 2),
                input_buffer.get_index(cursor + 3),
            ];
            cursor += 4;
            let y_slice = [
                input_buffer.get_index(cursor),
                input_buffer.get_index(cursor + 1),
                input_buffer.get_index(cursor + 2),
                input_buffer.get_index(cursor + 3),
            ];
            cursor += 4;
            let touch = &mut touch_buf[id as usize];
            enable_addition[id as usize] = true;
            touch.x = f32::from_le_bytes(x_slice);
            touch.y = f32::from_le_bytes(y_slice);
            if !touch.enable {
                touch.init_x = touch.x;
                touch.init_y = touch.y;
            }
            touch.enable = true;
        }
        touch_buf.iter_mut().enumerate().for_each(|(i, it)| {
            if !enable_addition[i] && it.enable {
                it.enable = false;
                it.touch_valid = true;
            }
            if enable_addition[i] {
                it.enable = true;
            }
        });
    });
}
