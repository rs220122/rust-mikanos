use crate::error::Error;

// CONFIG_ADDRESSレジスタのIOポートアドレス
const CONFIG_ADDRESS: u16 = 0x0cf8;
// CONFIG_DATAレジスタのIOポートアドレス
const CONFIG_DATA: u16 = 0x0cfc;

// デバイス管理用のグローバル変数
pub static mut DEVICES: [Device; 32] = [Device::default(); 32];
pub static mut NUM_DEVICES: usize = 0;
const DEVICES_LEN: usize = 32;

unsafe extern "C" {
    pub fn IoOut32(addr: u16, data: u32);
    pub fn IoIn32(addr: u16) -> u32;
}

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct Device {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub header_type: u8,
}

impl Device {
    pub const fn new(bus: u8, device: u8, function: u8, header_type: u8) -> Self {
        Self {
            bus,
            device,
            function,
            header_type,
        }
    }

    pub const fn default() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

// CONFIG_ADDRESS用の32bitの整数を生成する
pub fn make_address(bus: u8, device: u8, function: u8, reg_addr: u8) -> u32 {
    // bus: PCI伸ばす
    // deivce: busに接続されているPCIデバイスの番号
    // function: deviceで指定したPCIデバイスに実装亜sれているファンクション

    // bits分ビットシフトする関数
    let shl = |x: u8, bits: u8| return (x as u32) << bits;
    return shl(1, 31)
        | shl(bus, 16)
        | shl(device, 11)
        | shl(function, 8)
        | (reg_addr as u32 & 0xfc);
}

// CONFIG_ADDRESSレジスタに読み出したいPCIデバイスのアドレスを設定する。
pub fn write_address(address: u32) {
    unsafe { IoOut32(CONFIG_ADDRESS, address) }
}

// CONFIG_ADDRSSレジスタで設定したPCIデバイス空間にデータを書き込む
pub fn write_data(value: u32) {
    unsafe { IoOut32(CONFIG_DATA, value) }
}

// CONFIG_ADDRSSレジスタで設定したPCIデバイス空間のデータを取り出す
pub fn read_data() -> u32 {
    unsafe { IoIn32(CONFIG_DATA) }
}

// ベンダーIDを読み取る。PCIコンフィグレーション空間の最初の16bitに対応
pub fn read_vendor_id(bus: u8, device: u8, function: u8) -> u16 {
    write_address(make_address(bus, device, function, 0x00));
    return (read_data()) as u16;
}

pub fn read_device_id(bus: u8, device: u8, function: u8) -> u16 {
    write_address(make_address(bus, device, function, 0x00));
    return (read_data() >> 16) as u16;
}

pub fn read_header_type(bus: u8, device: u8, function: u8) -> u8 {
    // PCIコンフィグレーション空間のheader_typeを参照する。
    // 一般的なPCIデバイスは、0x00になっている。
    // bit7が、1の時は、マルチファンクションデバイス
    write_address(make_address(bus, device, function, 0x0c));
    return (((read_data() >> 16) as u16) & 0xff) as u8;
}

pub fn read_class_code(bus: u8, device: u8, function: u8) -> u32 {
    // PCIコンフレーション空間のクラスコード(0x0Bh-08h)の4バイト(32bit)
    // 31:24bitがbase_class
    // 23:16bitがsub class 細かいデバイス種別
    // 15:8bitはProgramming Interfaceで、0x20ならUSB2.0(EHCI), 0x30 USB3.0(xHCI)
    write_address(make_address(bus, device, function, 0x08));
    return read_data();
}

pub fn read_bus_numbers(bus: u8, device: u8, function: u8) -> u32 {
    write_address(make_address(bus, device, function, 0x18));
    return read_data();
}

pub fn is_single_function_device(header_type: u8) -> bool {
    return (header_type & 0x80) == 0;
}

fn add_device(bus: u8, device: u8, function: u8, header_type: u8) -> Result<(), Error> {
    unsafe {
        if NUM_DEVICES == DEVICES_LEN {
            return Err(Error::Full);
        }
        DEVICES[NUM_DEVICES] = Device {
            bus,
            device,
            function,
            header_type,
        };
        NUM_DEVICES += 1;
    }
    Ok(())
}

fn scan_function(bus: u8, device: u8, function: u8) -> Result<(), Error> {
    let header_type = read_header_type(bus, device, function);
    add_device(bus, device, function, header_type)?;

    // もしPCI-PCIブリッジならセカンダリバスに対して、scan_busを実行する。
    // PCI-PCIブリッジかを判定するには、base class, subclassを用いる
    //
    let class_code = read_class_code(bus, device, function);
    let base: u8 = (class_code >> 24) as u8;
    let sub: u8 = (class_code >> 16) as u8;

    if base == 0x06 && sub == 0x04 {
        let bus_numbers = read_bus_numbers(bus, device, function);
        let secondary_bus: u8 = (bus_numbers >> 8) as u8;
        return scan_bus(secondary_bus);
    }
    Ok(())
}

// 有効なfunctionを探し、見つけたらscanfunctionに処理を引き継ぐ
fn scan_device(bus: u8, device: u8) -> Result<(), Error> {
    scan_function(bus, device, 0)?;
    if is_single_function_device(read_header_type(bus, device, 0)) {
        return Ok(());
    }

    for function in 1..8 {
        if read_vendor_id(bus, device, function) == 0xffff {
            continue;
        }
        scan_function(bus, device, function)?;
    }
    Ok(())
}

// 指定したバス番号の各デバイスをスキャンする
// 有効なデバイスを見つけたらscan_deviceを実行する
fn scan_bus(bus: u8) -> Result<(), Error> {
    let max_bus: u8 = 32;
    for device in 0..max_bus {
        // デバイスがあるかどうかは、ファンクション0のvendor_idで判断
        // 0xffffの時は、ない。
        if read_vendor_id(bus, device, 0) == 0xffff {
            continue;
        }
        scan_device(bus, device)?;
    }
    Ok(())
}

pub fn scan_all_bus() -> Result<(), Error> {
    unsafe {
        NUM_DEVICES = 0;
    }
    // bus=0, device=0, function=0は、ホストブリッジと呼ばれ、
    // CPUとPCIデバイスが通信する時に通過
    // ホストブリッジのヘッダタイプを取得し、マルチファンクションデバイスか判定する。
    // マルチファンクションデバイスは、ファンクション0以外にも機能を機能を持っているPCIデバイス
    // シングルファンクションデバイスのホストブリッジは、バス0を担当するホストブリッジ
    let header_type = read_header_type(0, 0, 0);
    if is_single_function_device(header_type) {
        return scan_bus(0);
    }

    // マルチファンクションデバイスの処理
    // ホストブリッジが複数存在する。
    // ファンクション0のホストブリッジはバス0を担当、ファンクション1のホストブリッジはバス1を担当
    for function in 1..8 {
        if read_vendor_id(0, 0, function) == 0xffff {
            continue;
        }
        scan_bus(function)?;
    }
    Ok(())
}
