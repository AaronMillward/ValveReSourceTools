// use bitflags::bitflags;
// struct MDL {

// }

// bitflags! {
// 	struct StudioHeaderFlags: i32 {
// 		const AUTOGENERATED_HITBOX           = 1 << 0;  /* This flag is set if no hitbox information was specified */
// 		const USES_ENV_CUBEMAP               = 1 << 1;  /* This flag is set at loadtime, not mdl build time so that we don't have to rebuild models when we change materials. */
// 		const FORCE_OPAQUE                   = 1 << 2;  /* Use this when there are translucent parts to the model but we're not going to sort it. */
// 		const TRANSLUCENT_TWOPASS            = 1 << 3;  /* Use this when we want to render the opaque parts during the opaque pass and the translucent parts during the translucent pass. Added using $mostlyopaque to the QC. */
// 		const STATIC_PROP                    = 1 << 4;  /* This is set any time the .qc files has $staticprop in it. Means there's no bones and no transforms. */
// 		const USES_FB_TEXTURE                = 1 << 5;  /* This flag is set at loadtime, not mdl build time so that we don't have to rebuild models when we change materials. */
// 		const HASSHADOWLOD                   = 1 << 6;  /* This flag is set by studiomdl.exe if a separate "$shadowlod" entry was present for the .mdl (the shadow lod is the last entry in the lod list if present). */
// 		const USES_BUMPMAPPING               = 1 << 7;  /* This flag is set at loadtime, not mdl build time so that we don't have to rebuild models when we change materials. */
// 		const USE_SHADOWLOD_MATERIALS        = 1 << 8;  /* This flag is set when we should use the actual materials on the shadow LOD instead of overriding them with the default one (necessary for translucent shadows). */
// 		const OBSOLETE                       = 1 << 9;  /* This flag is set when we should use the actual materials on the shadow LOD instead of overriding them with the default one (necessary for translucent shadows). */
// 		const UNUSED                         = 1 << 10; /*  */
// 		const NO_FORCED_FADE                 = 1 << 11; /* This flag is set at mdl build time. */
// 		const FORCE_PHONEME_CROSSFADE        = 1 << 12; /* The npc will lengthen the viseme check to always include two phonemes. */
// 		const CONSTANT_DIRECTIONAL_LIGHT_DOT = 1 << 13; /* This flag is set when the .qc has $constantdirectionallight in it. If set, we use constantdirectionallightdot to calculate light intensity rather than the normal directional dot product. Only valid if STUDIOHDR_FLAGS_STATIC_PROP is also set. */
// 		const FLEXES_CONVERTED               = 1 << 14; /* Flag to mark delta flexes as already converted from disk format to memory format. */
// 		const BUILT_IN_PREVIEW_MODE          = 1 << 15; /* Indicates the studiomdl was built in preview mode (added with the -preview flag). */
// 		const AMBIENT_BOOST                  = 1 << 16; /* Ambient boost (runtime flag). */
// 		const DO_NOT_CAST_SHADOWS            = 1 << 17; /* Don't cast shadows from this model (useful on first-person models). */
// 		const CAST_TEXTURE_SHADOWS           = 1 << 18; /* Alpha textures should cast shadows in vrad on this model (ONLY prop_static!). Requires setup in the lights.rad file.  */
// 	}
// }

// #[repr(C, packed)]
// struct StudioMDLHeader {
// 	/* https://developer.valvesoftware.com/wiki/MDL */
// 	id : i32, /* Model format ID, such as "IDST" (0x49 0x44 0x53 0x54) */
// 	version : i32, /* Format version number, such as 48 (0x30,0x00,0x00,0x00) */
// 	checksum : i32, /* This has to be the same in the phy and vtx files to load! */
// 	name : [u8; 64], /* The internal name of the model, padding with null bytes. Typically "my_model.mdl" will have an internal name of "my_model"*/
// 	data_length : i32, /* Data size of MDL file in bytes. */

// 	/* A vector should be 12 bytes, three 4-byte float-values in a row. */
// 	eyeposition : glam::Vec3, /* Position of player viewpoint relative to model origin */
// 	illumposition : glam::Vec3, /* ?? Presumably the point used for lighting when per-vertex lighting is not enabled. */
// 	hull_min : glam::Vec3, /* Corner of model hull box with the least X/Y/Z values */
// 	hull_max : glam::Vec3, /* Opposite corner of model hull box */
// 	view_bbmin : glam::Vec3,
// 	view_bbmax : glam::Vec3,

// 	flags : i32, /* Binary flags in little-endian order. */
// 	/* ex (00000001,00000000,00000000,11000000) means flags for position 0, 30, and 31 are set. 
// 	Set model flags section for more information */

// 	/*
// 	* After this point, the header contains many references to offsets
// 	* within the MDL file and the number of items at those offsets.
// 	*
// 	* Offsets are from the very beginning of the file.
// 	* 
// 	* Note that indexes/counts are not always paired and ordered consistently.
// 	*/	

// 	// mstudiobone_t
// 	bone_count : i32,	// Number of data sections (of type mstudiobone_t)
// 	bone_offset : i32,	// Offset of first data section

// 	// mstudiobonecontroller_t
// 	bonecontroller_count : i32,
// 	bonecontroller_offset : i32,

// 	// mstudiohitboxset_t
// 	hitbox_count : i32,
// 	hitbox_offset : i32,

// 	// mstudioanimdesc_t
// 	localanim_count : i32,
// 	localanim_offset : i32,

// 	// mstudioseqdesc_t
// 	localseq_count : i32,
// 	localseq_offset : i32,

// 	activitylistversion : i32, // ??
// 	eventsindexed : i32,	// ??

// 	// VMT texture filenames
// 	// mstudiotexture_t
// 	texture_count : i32,
// 	texture_offset : i32,

// 	// This offset points to a series of ints.
// 		// Each int value, in turn, is an offset relative to the start of this header/the-file,
// 		// At which there is a null-terminated string.
// 	texturedir_count : i32,
// 	texturedir_offset : i32,

// 	// Each skin-family assigns a texture-id to a skin location
// 	skinreference_count : i32,
// 	skinrfamily_count : i32,
// 	skinreference_index : i32,

// 	// mstudiobodyparts_t
// 	bodypart_count : i32,
// 	bodypart_offset : i32,

// 		// Local attachment points		
// 	// mstudioattachment_t
// 	attachment_count : i32,
// 	attachment_offset : i32,

// 	// Node values appear to be single bytes, while their names are null-terminated strings.
// 	localnode_count : i32,
// 	localnode_index : i32,
// 	localnode_name_index : i32,

// 	// mstudioflexdesc_t
// 	flexdesc_count : i32,
// 	flexdesc_index : i32,

// 	// mstudioflexcontroller_t
// 	flexcontroller_count : i32,
// 	flexcontroller_index : i32,

// 	// mstudioflexrule_t
// 	flexrules_count : i32,
// 	flexrules_index : i32,

// 	// IK probably referse to inverse kinematics
// 	// mstudioikchain_t
// 	ikchain_count : i32,
// 	ikchain_index : i32,

// 	// Information about any "mouth" on the model for speech animation
// 	// More than one sounds pretty creepy.
// 	// mstudiomouth_t
// 	mouths_count : i32, 
// 	mouths_index : i32,

// 	// mstudioposeparamdesc_t
// 	localposeparam_count : i32,
// 	localposeparam_index : i32,

// 	/*
// 	* For anyone trying to follow along, as of this writing,
// 	* the next "surfaceprop_index" value is at position 0x0134 (308)
// 	* from the start of the file.
// 	*/

// 	// Surface property value (single null-terminated string)
// 	surfaceprop_index : i32,

// 	// Unusual: In this one index comes first, then count.
// 	// Key-value data is a series of strings. If you can't find
// 	// what you're interested in, check the associated PHY file as well.
// 	keyvalue_index : i32,
// 	keyvalue_count : i32,

// 	// More inverse-kinematics
// 	// mstudioiklock_t
// 	iklock_count : i32,
// 	iklock_index : i32,

// 	mass : f32, // Mass of object (4-bytes)
// 	contents : i32, // ??

// 	// Other models can be referenced for re-used sequences and animations
// 	// (See also: The $includemodel QC option.)
// 	// mstudiomodelgroup_t
// 	includemodel_count : i32,
// 	includemodel_index : i32,
	
// 	virtual_model : i32,	// Placeholder for mutable-void*

// 	// mstudioanimblock_t
// 	animblocks_name_index : i32,
// 	animblocks_count : i32,
// 	animblocks_index : i32,
	
// 	animblock_model : i32, // Placeholder for mutable-void*

// 	// Points to a series of bytes?
// 	bonetablename_index : i32,
	
// 	vertex_base : i32,	// Placeholder for void*
// 	offset_base : i32,	// Placeholder for void*
	
// 	// Used with $constantdirectionallight from the QC 
// 	// Model should have flag #13 set if enabled
// 	directionaldotproduct : u8,
	
// 	root_lod : u8,	// Preferred rather than clamped
	
// 	// 0 means any allowed, N means Lod 0 -> (N-1)
// 	num_allowed_root_lods : u8,	
	
// 	unused0 : u8, // ?? Struct padding?
// 	unused1 : i32, // ??
	
// 	// mstudioflexcontrollerui_t
// 	flexcontrollerui_count : i32,
// 	flexcontrollerui_index : i32,
	
// 	/**
// 	 * Offset for additional header information.
// 	 * May be zero if not present, or also 408 if it immediately 
// 	 * follows this studiohdr_t
// 	 */
// 	// studiohdr2_t
// 	studiohdr2index : i32,
	
// 	unused2 : i32, // ??
	
// 	/*
// 	 * As of this writing, the header is 408 bytes long in total
// 	 */
// 	/* ^ Aaron: I don't know why this comment is here because it's actually 400? */
// }