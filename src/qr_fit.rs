use image::{GenericImage, GenericImageView, ImageBuffer, RgbImage, Luma, imageops};

pub fn fit(qr: String) -> Vec<Vec<u8>> {
    let split_qr = qr.split('\n');

    let mut qr_arr = vec![];

    // Convert generated qrcode to 1s and 0s
    for line in split_qr {
        let line: Vec<u8> =
            line.chars().into_iter().map(|chr| {
                match chr {
                    ' ' => 0,
                    _ => 1
                }
            }).collect();
        
        qr_arr.push(line);
    }

    let mut perim = qr_arr.len();

    // Generate image
    let mut img = ImageBuffer::from_fn(perim as u32, perim as u32, |x, y| {
        match qr_arr[x as usize][y as usize] {
            1 => Luma([0u8]),
            _ => Luma([255u8])
        }   
    });

    let mut arr_final = vec![];
     
    // Resize to 62x62
    let img_resized = imageops::resize(&img, 62, 62, imageops::FilterType::Nearest);   

    // Convert Luma to 1s and 0s
    for x in 0..62 {
        let mut x_row = vec![];
        
        for y in 0..62 {
            x_row.push(
                match img_resized[(x, y)] {
                    Luma([0u8]) => 1,
                    _ => 0
                }
            );
        }
        arr_final.push(x_row);
    }
    
    arr_final
}
