// For format version 15!
// i have no use for this not sure why i spent so long making it...

#pragma endian big

struct Vec2f {
    float x;
    float y;
};

struct Vec2u {
    u32 x;
    u32 y;
};

struct Vec3f {
    float x;
    float y;
    float z;
};

struct Vec3u {
    u32 x;
    u32 y;
    u32 z;
};


enum Mode : u8 {
    Raster = 0,
    Vector = 1,
};

struct Supersample {
    u8 xy;
    u8 z;
};

struct VariableLayerHeight {
    bool enabled;
    float threshold;
    u8 max_layers;
    float exposure_s;
};

struct ElephantFootFixer {
    bool enabled;
    float inset_mm;
    float intensity;
};

struct PostProcessing {
    VariableLayerHeight variable_layer_height;
    ElephantFootFixer elephant_foot_fixer;
};

struct ExposureRemap {
    float start;
    float end;
    Vec2f control_a;
    Vec2f control_b;
};

struct ExposureConfig {
    float exposure_time;
    float exposure_delay;
    u8 pwm;
    float lift_distnace;
    float lift_speed;
    float retract_speed;
};

enum HeightTag : u8 {
    Layers = 0,
    Distance = 1
};

struct Height {
    HeightTag tag;

    if (tag == HeightTag::Layers) {
        u32 layers;
     } else {
        float distance_mm;
     }
};

struct SliceConfig {
    Mode mode;
    Supersample supersample;
    ExposureRemap exposure_remap;
    Vec2u platform_res;
    Vec3f platform_size;
    float slice_height;
    ExposureConfig exposure_config;
    ExposureConfig first_exposure_config;
    Height first_layers;
    Height transition_layers;
};

enum UnitTag : u8 {
    Custom = 0,
    Millimeters = 1,
    Centimeters = 2,
    Meters = 3,
    Inches = 4,
};

struct Unit {
    UnitTag tag;
    if (tag == UnitTag::Custom) { float custom_factor; }
};

struct Model {
    u32 mesh;
    bool collection;
    if (collection) { u32 collection_id; }

    u32 name_len;
    char name[name_len];
    Unit unit;
    Vec3f color;
    u8 exposure;
    bool hidden;

    Vec3f position;
    Vec3f scale;
    Vec3f rotation;
};

struct Mesh {
    u32 vert_count;
    Vec3f verts[vert_count];

    u32 face_count;
    Vec3u faces[face_count];
};

struct Collection {
    u32 id;
    u32 name_len;
    char name[name_len];
    bool collapsed;
};

struct Project {
    u16 version;
    SliceConfig slice_config;
    PostProcessing post_processing;

    u32 model_count;
    Model models[model_count];

    u32 mesh_count;
    Mesh meshes[mesh_count] [[color("FFFFFF")]];

    u32 collection_count;
    Collection collections[collection_count];
};

Project project @ 0x00;
