#include <stdint.h>
#include <stdbool.h>

// use the indirect function table to get emscripten to import the function table
// extern void (*abc)();
// void abcde() { abc(); }
// the resulting line of code is this
// (import "env" "__indirect_function_table" (table (;0;) 0 funcref))

typedef uint64_t TurtleWho;
typedef uint64_t BreedId;
typedef double Float;
typedef char RustString[12];

typedef struct {
	Float x;
	Float y;
} Point;

typedef struct {
	uint32_t x;
	uint32_t y;
} PointInt;

typedef struct {
	uint8_t buffer_idx;
	uint8_t field_idx;
} AgentFieldDescriptor;

typedef union{
	struct {
		uint32_t index;
		uint16_t gen;
	} gen_index;
	uint64_t raw;
} TurtleId;

typedef int32_t PatchId;

typedef struct {
	TurtleWho who;
	BreedId breed;
	RustString shape_name;
	Float color;
	RustString label;
	Float label_color;
	bool hidden;
	Float size;
} TurtleBaseData;

typedef struct {
	Point position;
	RustString plabel;
	Float plabel_color;
} PatchBaseData;

typedef struct {} TurtleSet;

typedef struct {} TurtleIterator;

typedef struct {} PatchIterator;

// relating to the execution context and workspace data structure

typedef struct {} Context;

typedef struct {} Updater;

typedef struct {} Workspace;

typedef struct {} World;

#define OFFSET_WORLD_TO_TURTLES         0x00000000
#define OFFSET_WORLD_TO_PATCHES         0x00000148
#define OFFSET_WORLD_TO_TOPOLOGY        0x00000228
#define OFFSET_WORLD_TO_TICK_COUNTER    0x00000270
#define OFFSET_PATCHES_TO_DATA          0x00000020
#define SIZE_OF_ROW_BUFFER              0x00000028
#define OFFSET_TOPOLOGY_TO_MAX_PXCOR    0x00000008
#define OFFSET_TOPOLOGY_TO_MAX_PYCOR    0x00000020
#define OFFSET_TURTLES_TO_DATA          0x00000010
#define OFFSET_CONTEXT_TO_WORKSPACE     0x00000000
#define OFFSET_CONTEXT_TO_UPDATER       0x00000008
#define OFFSET_WORKSPACE_TO_WORLD       0x00000000

static inline Updater *context_to_updater(Context *context) {
	return (Updater *)((char *)context + OFFSET_CONTEXT_TO_UPDATER);
}

static inline World *context_to_world(Context *context) {
	Workspace *workspace = *(Workspace **)((char *)context + OFFSET_CONTEXT_TO_WORKSPACE);
	return (World *)((char *)workspace + OFFSET_WORKSPACE_TO_WORLD);
}

static inline void *world_to_turtle_data(World *world, int buffer_idx) {
	return *(void **)((char *)world + OFFSET_WORLD_TO_TURTLES + OFFSET_TURTLES_TO_DATA + buffer_idx * SIZE_OF_ROW_BUFFER);
}

static inline void *world_to_patch_data(World *world, int buffer_idx) {
	return *(void **)((char *)world + OFFSET_WORLD_TO_PATCHES + OFFSET_PATCHES_TO_DATA + buffer_idx * SIZE_OF_ROW_BUFFER);
}

static inline Float world_to_max_pxcor(World *world) {
	return *(Float *)((char *)world + OFFSET_WORLD_TO_TOPOLOGY + OFFSET_TOPOLOGY_TO_MAX_PXCOR);
}

static inline Float world_to_max_pycor(World *world) {
	return *(Float *)((char *)world + OFFSET_WORLD_TO_TOPOLOGY + OFFSET_TOPOLOGY_TO_MAX_PYCOR);
}

void oxitortoise_update_turtle(Updater *updater, World *world, TurtleId turtle_id, uint16_t flags);
void oxitortoise_update_patch(Updater *updater, World *world, PatchId patch_id, uint8_t flags);
void oxitortoise_update_tick(Updater *updater, Float tick);

bool oxitortoise_is_nan(double value);
Float oxitortoise_round(Float value);

void oxitortoise_clear_all(Context *context);
void oxitortoise_reset_ticks(World *world);
Float oxitortoise_get_ticks(World *world);
void oxitortoise_advance_tick(World *world);

TurtleIterator *oxitortoise_create_turtles(Context *context, BreedId breed, uint64_t count, Point position);

TurtleIterator *oxitortoise_make_all_turtles_iter(Context *context);

TurtleId oxitortoise_next_turtle_from_iter(TurtleIterator *iter);
void oxitortoise_drop_turtle_iter(TurtleIterator *iter);

PatchIterator *oxitortoise_make_all_patches_iter(Context *context);

PatchId oxitortoise_next_patch_from_iter(PatchIterator *iter);
void oxitortoise_drop_patch_iter(PatchIterator *iter);

Float oxitortoise_distance_euclidean_no_wrap(Point a, Point b);
Point oxitortoise_offset_distance_by_heading(World *world, Point position, Float heading, Float distance);
PatchId oxitortoise_patch_at(World *world, PointInt position);
Float oxitortoise_normalize_heading(Float heading);

void oxitortoise_diffuse_8(World *world, AgentFieldDescriptor field, Float diffusion_rate);

Float oxitortoise_scale_color(Float color, Float value, Float min, Float max);

uint32_t oxitortoise_next_int(Context *context, uint32_t max);

BreedId oxitortoise_get_default_turtle_breed(Context *context);

typedef struct {
	uint8_t occupancy_bitfield[1];
	TurtleBaseData base_data;
	Float heading;
	Point position;
} TurtleGroup0;
typedef struct {
	uint8_t occupancy_bitfield[1];
	PatchBaseData base_data;
	Float food;
	bool nest;
	Float nest_scent;
	Float food_source_number;
} PatchGroup0;
typedef struct {
	Float pcolor;
} PatchGroup1;
typedef struct {
	Float chemical;
} PatchGroup2;

// --- Patch field indices ---
#define PATCH_CHEMICAL (AgentFieldDescriptor){.buffer_idx = 2, .field_idx = 0}
#define PATCH_FOOD (AgentFieldDescriptor){.buffer_idx = 0, .field_idx = 1}
#define PATCH_NEST (AgentFieldDescriptor){.buffer_idx = 0, .field_idx = 2}
#define PATCH_NEST_SCENT (AgentFieldDescriptor){.buffer_idx = 0, .field_idx = 3}
#define PATCH_FOOD_SOURCE_NUMBER (AgentFieldDescriptor){.buffer_idx = 0, .field_idx = 4}

// --- Color constants ---
#define COLOR_RED 15.0
#define COLOR_ORANGE 25.0
#define COLOR_GREEN 65.0
#define COLOR_CYAN 85.0
#define COLOR_SKY 95.0
#define COLOR_BLUE 105.0
#define COLOR_VIOLET 115.0

// --- Point constants ---
#define POINT_ORIGIN (Point){.x = 0.0, .y = 0.0}

// --- Heading constants ---
#define HEADING_MAX 360.0

// --- Updater flags ---
#define FLAG_BREED 1 << 0
#define FLAG_COLOR 1 << 1
#define FLAG_HEADING 1 << 2
#define FLAG_LABEL_COLOR 1 << 3
#define FLAG_LABEL 1 << 4
#define FLAG_HIDDEN 1 << 5
#define FLAG_PEN_SIZE 1 << 6
#define FLAG_PEN_MODE 1 << 7
#define FLAG_SHAPE 1 << 8
#define FLAG_SIZE 1 << 9
#define FLAG_POSITION 1 << 10
#define FLAG_PCOLOR 1 << 0
#define FLAG_PLABEL 1 << 1
#define FLAG_PLABEL_COLOR 1 << 2

void recolor_patch(Context *context, PatchId patch_id) {
	World *world = context_to_world(context);
	Updater *updater = context_to_updater(context);

	PatchGroup0 *patch0 = (PatchGroup0 *)world_to_patch_data(world, 0) + patch_id;
	PatchGroup1 *patch1 = (PatchGroup1 *)world_to_patch_data(world, 1) + patch_id;
	PatchGroup2 *patch2 = (PatchGroup2 *)world_to_patch_data(world, 2) + patch_id;
	if (patch0->nest) {
		patch1->pcolor = COLOR_VIOLET;
	} else if (patch0->food > 0.0) {
		if (patch0->food_source_number == 1.0) {
			patch1->pcolor = COLOR_CYAN;
		} else if (patch0->food_source_number == 2.0) {
			patch1->pcolor = COLOR_SKY;
		} else if (patch0->food_source_number == 3.0) {
			patch1->pcolor = COLOR_BLUE;
		}
	} else {
		// scale-color green chemical 0.1 5
		patch1->pcolor = oxitortoise_scale_color(COLOR_GREEN, patch2->chemical, 0.1, 5.0);
	}
	oxitortoise_update_patch(updater, world, patch_id, FLAG_PCOLOR);
}

Float chemical_at_angle(World *world, Point position, Float heading, Float angle) {
	Float real_heading = oxitortoise_normalize_heading(heading + angle);
	Point point_ahead = oxitortoise_offset_distance_by_heading(world, position, real_heading, 1.0);
	// assume that None is represented by both coordinates being NaN
	if (oxitortoise_is_nan(point_ahead.x)) {
		return 0.0;
	}

	PointInt point_ahead_int = (PointInt){.x = (uint32_t)oxitortoise_round(point_ahead.x), .y = (uint32_t)oxitortoise_round(point_ahead.y)};
	PatchId patch_id = oxitortoise_patch_at(world, point_ahead_int);

	PatchGroup2 *patch2 = (PatchGroup2 *)world_to_patch_data(world, 2) + patch_id;
	return patch2->chemical;
}

void uphill_chemical(World *world, Point position, Float *heading) {
	Float chemical_ahead = chemical_at_angle(world, position, *heading, 0.0);
	Float chemical_right = chemical_at_angle(world, position, *heading, 45.0);
	Float chemical_left = chemical_at_angle(world, position, *heading, -45.0);
	if (chemical_right > chemical_ahead || chemical_left > chemical_ahead) {
		if (chemical_right > chemical_left) {
			Float new_heading = oxitortoise_normalize_heading(*heading + 45.0);
			*heading = new_heading;
		} else {
			Float new_heading = oxitortoise_normalize_heading(*heading - 45.0);
			*heading = new_heading;
		}
	}
}

Float nest_scent_at_angle(World *world, Point position, Float heading, Float angle) {
	Float real_heading = oxitortoise_normalize_heading(heading + angle);
	Point point_ahead = oxitortoise_offset_distance_by_heading(world, position, real_heading, 1.0);
	// assume that None is represented by both coordinates being NaN
	if (oxitortoise_is_nan(point_ahead.x)) {
		return 0.0;
	}

	PointInt point_ahead_int = (PointInt){.x = (uint32_t)oxitortoise_round(point_ahead.x), .y = (uint32_t)oxitortoise_round(point_ahead.y)};
	PatchId patch_id = oxitortoise_patch_at(world, point_ahead_int);

	PatchGroup0 *patch0 = (PatchGroup0 *)world_to_patch_data(world, 0) + patch_id;
	return patch0->nest_scent;
}

void uphill_nest_scent(World *world, Point position, Float *heading) {
	Float scent_ahead = nest_scent_at_angle(world, position, *heading, 0.0);
	Float scent_right = nest_scent_at_angle(world, position, *heading, 45.0);
	Float scent_left = nest_scent_at_angle(world, position, *heading, -45.0);
	if (scent_right > scent_ahead || scent_left > scent_ahead) {
		if (scent_right > scent_left) {
			Float new_heading = oxitortoise_normalize_heading(*heading + 45.0);
			*heading = new_heading;
		} else {
			Float new_heading = oxitortoise_normalize_heading(*heading - 45.0);
			*heading = new_heading;
		}
	}
}

void setup(Context *context) {
	Updater *updater = context_to_updater(context);
	World *world = context_to_world(context);

	// clear-all
	oxitortoise_clear_all(context);

	// create-turtles
	{
		TurtleIterator *iter = oxitortoise_create_turtles(
			context,
			oxitortoise_get_default_turtle_breed(context),
			125,
			POINT_ORIGIN
		);
		TurtleId next_turtle;
		while ((next_turtle = oxitortoise_next_turtle_from_iter(iter)).raw != 0) {
			// since the turtles were just created and we know they don't die,
			// we don't need to check the generation
			TurtleGroup0 *turtle_data = (TurtleGroup0 *)world_to_turtle_data(world, 0) + next_turtle.gen_index.index;
			TurtleBaseData *base_data = &turtle_data->base_data;

			base_data->size = 2.0;
			base_data->color = 15.0;

			// TODO send update to the updater
			oxitortoise_update_turtle(updater, world, next_turtle, FLAG_COLOR | FLAG_SIZE);
		}
		oxitortoise_drop_turtle_iter(iter);
	}

	// setup-patches
	{
		PatchIterator *iter = oxitortoise_make_all_patches_iter(context);
		PatchId next_patch;
		while ((next_patch = oxitortoise_next_patch_from_iter(iter)) != (PatchId)(-1)) {
			// calculate distancexy 0 0
			PatchGroup0 *patch = (PatchGroup0 *)world_to_patch_data(world, 0) + next_patch;
			Point position = patch->base_data.position;
			Float distance = oxitortoise_distance_euclidean_no_wrap(position, POINT_ORIGIN);

			// set nest? (distancexy 0 0) < 5
			patch->nest = distance < 5.0;

			// set nest-scent 200 - distancexy 0 0
			patch->nest_scent = 200.0 - distance;

			// setup-food
			{
				Float max_pxcor = world_to_max_pxcor(world);
				Float max_pycor = world_to_max_pycor(world);

				// if (distancexy (0.6 * max-pxcor) 0) < 5 [ set food-source-number 1 ]
				{
					Float distance = oxitortoise_distance_euclidean_no_wrap(position, (Point){.x = 0.6 * max_pxcor, .y = 0.0});
					if (distance < 5.0) {
						patch->food_source_number = 1.0;
					}
				}

				// if (distancexy (-0.6 * max-pxcor) (-0.6 * max-pycor)) < 5 [ set food-source-number 2 ]
				{
					Float distance = oxitortoise_distance_euclidean_no_wrap(position, (Point){.x = -0.6 * max_pxcor, .y = -0.6 * max_pycor});
					if (distance < 5.0) {
						patch->food_source_number = 2.0;
					}
				}

				// if (distancexy (-0.8 * max-pxcor) (0.8 * max-pycor)) < 5 [ set food-source-number 3 ]
				{
					Float distance = oxitortoise_distance_euclidean_no_wrap(position, (Point){.x = -0.8 * max_pxcor, .y = 0.8 * max_pycor});
					if (distance < 5.0) {
						patch->food_source_number = 3.0;
					}
				}

				// if food-source-number > 0 [ set food one-of [1 2] ]
				{
					if (patch->food_source_number > 0.0) {
						uint32_t rand_index = oxitortoise_next_int(context, 2);
						patch->food = rand_index == 0 ? 1.0 : 2.0;
					}
				}

				// recolor-patch
				recolor_patch(context, next_patch);
			}
		}
		oxitortoise_drop_patch_iter(iter);
	}

	// reset-ticks
	oxitortoise_reset_ticks(world);
	oxitortoise_update_tick(updater, oxitortoise_get_ticks(world));
}

void shim_setup(Context *context, void *args) {
	setup(context);
}

void go(Context *context) {
	Updater *updater = context_to_updater(context);
	World *world = context_to_world(context);

	// ask turtles
	TurtleIterator *iter = oxitortoise_make_all_turtles_iter(context);
	TurtleId next_turtle;
	while ((next_turtle = oxitortoise_next_turtle_from_iter(iter)).raw != 0) {
		// if who >= ticks [ stop ]
		TurtleGroup0 *turtle0 = (TurtleGroup0 *)world_to_turtle_data(world, 0) + next_turtle.gen_index.index;
		if (turtle0->base_data.who >= oxitortoise_get_ticks(world)) {
			continue;
		}

		// cache some variables for later use
		Point *position = &turtle0->position;
		Float *heading = &turtle0->heading;

		// ifelse color = red
		if (turtle0->base_data.color == COLOR_RED) {
			// look-for-food
			{
				PatchId patch_here_id = oxitortoise_patch_at(world, (PointInt){.x = (uint32_t)oxitortoise_round(position->x), .y = (uint32_t)oxitortoise_round(position->y)});
				PatchGroup0 *patch_here = (PatchGroup0 *)world_to_patch_data(world, 0) + patch_here_id;

				// if food > 0
				if (patch_here->food > 0.0) {
					// set color orange + 1
					turtle0->base_data.color = COLOR_ORANGE + 1.0;

					// set food food - 1
					patch_here->food -= 1.0;

					// rt 180
					*heading = oxitortoise_normalize_heading(*heading + 180.0);

					// stop
					oxitortoise_update_turtle(updater, world, next_turtle, FLAG_POSITION | FLAG_HEADING | FLAG_COLOR);
					continue;
				}

				// if (chemical >= 0.05) and (chemical < 2)
				PatchGroup2 *patch2_here = (PatchGroup2 *)world_to_patch_data(world, 2) + patch_here_id;
				Float chemical = patch2_here->chemical;
				if (chemical >= 0.05 && chemical < 2.0) {
					// uphill-chemical
					uphill_chemical(world, *position, heading);
				}
			}
		} else {
			// return-to-nest
			{
				PatchId patch_here_id = oxitortoise_patch_at(world, (PointInt){.x = (uint32_t)oxitortoise_round(position->x), .y = (uint32_t)oxitortoise_round(position->y)});
				PatchGroup0 *patch_here = (PatchGroup0 *)world_to_patch_data(world, 0) + patch_here_id;

				// ifelse nest?
				if (patch_here->nest) {
					// set color red
					turtle0->base_data.color = COLOR_RED;

					// rt 180
					*heading = oxitortoise_normalize_heading(*heading + 180.0);
				} else {
					// set chemical chemical + 60
					PatchGroup2 *patch2_here = (PatchGroup2 *)world_to_patch_data(world, 2) + patch_here_id;
					patch2_here->chemical += 60.0;

					// uphill-nest-scent
					uphill_nest_scent(world, *position, heading);
				}
			}
		}

		// wiggle
		{
			// rt random 40
			Float rand_result = (Float)oxitortoise_next_int(context, 40);
			*heading = oxitortoise_normalize_heading(*heading + rand_result);

			// lt random 40
			rand_result = (Float)oxitortoise_next_int(context, 40);
			*heading = oxitortoise_normalize_heading(*heading - rand_result);

			// if not can-move? 1 [ rt 180 ]
			Point point_ahead = oxitortoise_offset_distance_by_heading(world, *position, *heading, 1.0);
			if (oxitortoise_is_nan(point_ahead.x)) {
				*heading = oxitortoise_normalize_heading(*heading + 180.0);
			}
		}

		// fd 1
		Point new_position = oxitortoise_offset_distance_by_heading(world, *position, *heading, 1.0);
		if (!oxitortoise_is_nan(new_position.x)) {
			*position = new_position;
		}

		oxitortoise_update_turtle(updater, world, next_turtle, FLAG_POSITION | FLAG_HEADING | FLAG_COLOR);
	}
	oxitortoise_drop_turtle_iter(iter);

	// diffuse chemical (diffusion-rate / 100)
	oxitortoise_diffuse_8(world, PATCH_CHEMICAL, 0.5);

	// ask patches
	PatchIterator *patch_iter = oxitortoise_make_all_patches_iter(context);
	PatchId next_patch;
	while ((next_patch = oxitortoise_next_patch_from_iter(patch_iter)) != (PatchId)(-1)) {
		// set chemical chemical * (100 - evaporation-rate) / 100
		PatchGroup2 *patch2 = (PatchGroup2 *)world_to_patch_data(world, 2) + next_patch;
		patch2->chemical *= 0.9;

		// recolor-patch
		recolor_patch(context, next_patch);
	}
	oxitortoise_drop_patch_iter(patch_iter);

	// advance-tick
	oxitortoise_advance_tick(world);
	oxitortoise_update_tick(updater, oxitortoise_get_ticks(world));
}

void shim_go(Context *context, void *args) {
	go(context);
}

