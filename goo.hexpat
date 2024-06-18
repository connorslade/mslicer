#pragma endian big

struct Header {
    char version[0x04];
    u8 magic[0x08];
    char software_info[0x20];
    char software_version[0x18];
    char file_time[0x18];
    char printer_name[0x20];
    char printer_type[0x20];
    char profile_name[0x20];
    u16 anti_aliasing;
    u16 grey_level;
    u16 blur_level;
    u8 small_preview[0x6920];
    u8 delimiter_1[0x02];
    u8 u8_array_at_0x69E4[0x29108];
    u8 delimiter_2[0x02];
    u32 total_layers;
    u16 x_resolution;
    u16 y_resolution;
    bool x_mirror;
    bool y_mirror;
    float x_size;
    float y_size;
    float z_size;
    float layer_thickness;
    float common_exposure_time;
    bool exposure_delay_mode;
    float turn_off_time;
    float bottom_before_list_time;
    float bottom_after_list_time;
    float bottom_after_retract_time;
    float before_lift_time;
    float after_lift_time;
    float after_retract_time;
    float bottom_exposure_time;
    u32 bottom_layers;
    float bottom_lift_distance;
    float bottom_lift_speed;
    float lift_distance;
    float lift_speed;
    float bottom_retract_distance;
    float bottom_retract_speed;
    float retract_distance;
    float retract_speed;
    float bottom_second_lift_distance;
    float bottom_second_lift_speed;
    float second_lift_distance;
    float second_lift_speed;
    float bottom_second_retract_distance;
    float bottom_second_retract_speed;
    float second_retract_distance;
    float second_retract_speed;
    u16 bottom_light_pwm;
    u16 light_pwm;
    bool advance_mode;
    u32 printing_time;
    float total_volume;
    float total_weight;
    float total_price;
    char price_unit[0x08];
    u32 offset_of_layer_content;
    bool gray_scale_level;
    u16 transition_layers;
};

struct ImageData {
    u32 data_size;
    u8 magic;
    u8 data[data_size - 2];
    u8 checksum;
    u8 delimiter[0x02];
};

struct LayerContent {
    u16 pause;
    float pause_pos_z;
    float layer_pos_z;
    float layer_exposure_time;
    float layer_off_time;
    float before_lift_time;
    float after_lift_time;
    float after_retract_time;
    float lift_distance;
    float lift_speed;
    float second_lift_distance;
    float second_lift_speed;
    float retract_distance;
    float retract_speed;
    float second_retract_distance;
    float second_retract_speed;
    u16 light_pwm;
    u8 delimiter[0x02];
    
    ImageData image;
};

struct File {
    Header header;
    LayerContent layers[header.total_layers];
    u8 ending_string[0x0B];
};

File file @ 0x00;