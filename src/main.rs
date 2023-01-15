mod window;
use window::run;

fn main() {
    pollster::block_on(run());
}
