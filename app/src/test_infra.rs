extern crate std;

use crate::interface::DisplayColor;

// Taken from stdext: https://docs.rs/stdext/0.3.3/src/stdext/macros.rs.html#63-74
macro_rules! function_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        // `3` is the length of the `::f`.
        &name[..name.len() - 3]
    }};
}
pub(crate) use function_name;

pub(crate) type SimDisplay = embedded_graphics_simulator::SimulatorDisplay<DisplayColor>;

pub(crate) fn assert_snapshot(test_name: &str, display: SimDisplay) {
    let test_image_path = std::format!("snapshots/{test_name}.test.png");
    let golden_image_path = std::format!("snapshots/{test_name}.golden.png");

    let golden = std::fs::read(&golden_image_path);
    match golden {
        Ok(golden_image) => {
            // There is an existing golden image. Save our current image for reference by
            // the user.
            display
                .to_rgb_output_image(&core::default::Default::default())
                .save_png(&test_image_path)
                .unwrap();

            // Now compare the test image to the golden image.
            // Kindof hacky to read this back in right after we wrote it out, but
            // meh good enough for now.
            let test_image = std::fs::read(&test_image_path).unwrap();
            assert_eq!(
                test_image, golden_image,
                "{test_image_path} does not match {golden_image_path}"
            );
        }
        Err(_) => {
            // There is no golden image.
            //
            // This is either a new test, or the user has deleted the golden image
            // on purpose. In either case, save the current result as the golden image.
            display
                .to_rgb_output_image(&core::default::Default::default())
                .save_png(&golden_image_path)
                .unwrap();
        }
    }
}
