#[cfg(test)]
mod tests {
    use build_printers::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_round_bed_detection() {
        let printable_area = [
            "149.429x13.0734",
            "147.721x26.0472",
            "144.889x38.8229",
            "140.954x51.303",
            "135.946x63.3927",
            "129.904x75",
            "122.873x86.0365",
            "114.907x96.4181",
            "106.066x106.066",
            "96.4181x114.907",
            "86.0365x122.873",
            "75x129.904",
            "63.3927x135.946",
            "51.303x140.954",
            "38.8229x144.889",
            "26.0472x147.721",
            "13.0734x149.429",
            "9.18485e-15x150",
            "-13.0734x149.429",
            "-26.0472x147.721",
            "-38.8229x144.889",
            "-51.303x140.954",
            "-63.3927x135.946",
            "-75x129.904",
            "-86.0365x122.873",
            "-96.4181x114.907",
            "-106.066x106.066",
            "-114.907x96.4181",
            "-122.873x86.0365",
            "-129.904x75",
            "-135.946x63.3927",
            "-140.954x51.303",
            "-144.889x38.8229",
            "-147.721x26.0472",
            "-149.429x13.0734",
            "-150x1.83697e-14",
            "-149.429x-13.0734",
            "-147.721x-26.0472",
            "-144.889x-38.8229",
            "-140.954x-51.303",
            "-135.946x-63.3927",
            "-129.904x-75",
            "-122.873x-86.0365",
            "-114.907x-96.4181",
            "-106.066x-106.066",
            "-96.4181x-114.907",
            "-86.0365x-122.873",
            "-75x-129.904",
            "-63.3927x-135.946",
            "-51.303x-140.954",
            "-38.8229x-144.889",
            "-26.0472x-147.721",
            "-13.0734x-149.429",
            "-2.75546e-14x-150",
            "13.0734x-149.429",
            "26.0472x-147.721",
            "38.8229x-144.889",
            "51.303x-140.954",
            "63.3927x-135.946",
            "75x-129.904",
            "86.0365x-122.873",
            "96.4181x-114.907",
            "106.066x-106.066",
            "114.907x-96.4181",
            "122.873x-86.0365",
            "129.904x-75",
            "135.946x-63.3927",
            "140.954x-51.303",
            "144.889x-38.8229",
            "147.721x-26.0472",
            "149.429x-13.0734",
            "150x-3.67394e-14",
        ];
        let mut machine = OrcaFdmMachine::default();

        let pa = printable_area
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        machine.printable_area = Some(pa);
        machine.printable_height = Some("410.0".to_string());

        let bv = machine.get_build_volume().unwrap();

        assert_eq!(bv.round, true);
        assert_eq!(bv.center, true);
        assert_eq!(bv.height, 410);
        assert_eq!(bv.diameter, Some(150));
        assert_eq!(bv.width, None);
        assert_eq!(bv.depth, None);
    }

    #[test]
    fn test_square_bed_detection() {
        let printable_area = ["0x0", "210x0", "210x210", "0x210"];
        let mut machine = OrcaFdmMachine::default();

        let pa = printable_area
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        machine.printable_area = Some(pa);
        machine.printable_height = Some("205".to_string());

        let bv = machine.get_build_volume().unwrap();

        assert_eq!(bv.round, false);
        assert_eq!(bv.center, false);
        assert_eq!(bv.height, 205);
        assert_eq!(bv.width, Some(210));
        assert_eq!(bv.depth, Some(210));
        assert_eq!(bv.diameter, None);
    }

    #[test]
    fn test_rectangle_bed_detection() {
        let printable_area = ["0x0", "220x0", "220x210", "0x210"];
        let mut machine = OrcaFdmMachine::default();

        let pa = printable_area
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        machine.printable_area = Some(pa);
        machine.printable_height = Some("205".to_string());

        let bv = machine.get_build_volume().unwrap();

        assert_eq!(bv.round, false);
        assert_eq!(bv.center, false);
        assert_eq!(bv.height, 205);
        assert_eq!(bv.width, Some(220));
        assert_eq!(bv.depth, Some(210));
        assert_eq!(bv.diameter, None);
    }
}
