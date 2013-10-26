use hl::*;

pub fn create_compute_context() -> Result<(Device, Context, CommandQueue), ~str>
{
    let platforms = get_platforms();
    if platforms.len() == 0 {
        return Err(~"No platform found");
    }

    let devices = platforms[0].get_devices();
    if devices.len() == 0 {
        return Err(~"No devices found");
    }

    let context = devices[0].create_context();
    let queue = context.create_command_queue(&devices[0]);

    Ok((devices[0], context, queue))
}