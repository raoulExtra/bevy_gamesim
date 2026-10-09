import bpy
import math
import os

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(ROOT, "runtime", "munich_sbahn")


def material(name, color, roughness=0.45, metallic=0.0):
    mat = bpy.data.materials.new(name)
    mat.diffuse_color = (*color, 1.0)
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    bsdf.inputs["Base Color"].default_value = (*color, 1.0)
    bsdf.inputs["Roughness"].default_value = roughness
    bsdf.inputs["Metallic"].default_value = metallic
    return mat


WHITE = None
SILVER = None
RED = None
RED_DARK = None
WINDOW = None
WINDOW_HIGHLIGHT = None
BLACK = None
LIGHT = None


def init_materials():
    global WHITE, SILVER, RED, RED_DARK, WINDOW, WINDOW_HIGHLIGHT, BLACK, LIGHT
    WHITE = material("S-Bahn ivory", (0.78, 0.80, 0.78), 0.32, 0.05)
    SILVER = material("S-Bahn silver", (0.38, 0.42, 0.42), 0.3, 0.35)
    RED = material("Munich red", (0.64, 0.025, 0.018), 0.3, 0.05)
    RED_DARK = material("Red shadow", (0.30, 0.008, 0.006), 0.38, 0.02)
    WINDOW = material("Smoked windows", (0.012, 0.035, 0.052), 0.18, 0.15)
    WINDOW_HIGHLIGHT = material("Window reflection", (0.08, 0.20, 0.24), 0.15, 0.2)
    BLACK = material("Rubber and bogies", (0.018, 0.022, 0.022), 0.8, 0.0)
    LIGHT = material("Headlights", (0.95, 0.92, 0.70), 0.22, 0.0)


def box(name, location, dimensions, mat, bevel=0.0):
    bpy.ops.mesh.primitive_cube_add(location=location)
    obj = bpy.context.object
    obj.name = name
    obj.dimensions = dimensions
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    obj.data.materials.append(mat)
    if bevel:
        modifier = obj.modifiers.new("Soft manufactured edges", "BEVEL")
        modifier.width = bevel
        modifier.segments = 2
        modifier.limit_method = "ANGLE"
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.modifier_apply(modifier=modifier.name)
    return obj


def wedge(name, x_front, x_back, width_front, width_back, z_bottom, z_top, mat):
    vertices = []
    for x, width in ((x_front, width_front), (x_back, width_back)):
        vertices.extend(
            [
                (x, -width / 2, z_bottom),
                (x, width / 2, z_bottom),
                (x, -width / 2, z_top),
                (x, width / 2, z_top),
            ]
        )
    faces = [
        (0, 1, 3, 2),
        (4, 6, 7, 5),
        (0, 4, 5, 1),
        (2, 3, 7, 6),
        (0, 2, 6, 4),
        (1, 5, 7, 3),
    ]
    mesh = bpy.data.meshes.new(name + " mesh")
    mesh.from_pydata(vertices, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(mat)
    bevel = obj.modifiers.new("Cab edge softening", "BEVEL")
    bevel.width = 0.08
    bevel.segments = 2
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.modifier_apply(modifier=bevel.name)
    return obj


def cylinder(name, location, radius, depth, mat, rotation=(math.pi / 2, 0, 0)):
    bpy.ops.mesh.primitive_cylinder_add(
        vertices=16,
        radius=radius,
        depth=depth,
        location=location,
        rotation=rotation,
    )
    obj = bpy.context.object
    obj.name = name
    obj.data.materials.append(mat)
    return obj


def side_windows(prefix, x_centers, z=2.25):
    for side in (-1, 1):
        y = side * 1.31
        for index, x in enumerate(x_centers):
            box(f"{prefix} side window {side} {index}", (x, y, z), (1.35, 0.055, 0.82), WINDOW, 0.08)
            box(
                f"{prefix} window reflection {side} {index}",
                (x - 0.28, y + side * 0.035, z + 0.20),
                (0.28, 0.012, 0.08),
                WINDOW_HIGHLIGHT,
                0.02,
            )


def doors(prefix, x_centers):
    for side in (-1, 1):
        y = side * 1.325
        for index, x in enumerate(x_centers):
            box(f"{prefix} door {side} {index}", (x, y, 1.52), (0.82, 0.045, 1.65), SILVER, 0.04)
            box(f"{prefix} door window {side} {index}", (x, y + side * 0.03, 2.19), (0.58, 0.055, 0.68), WINDOW, 0.05)


def underframe(prefix):
    box(f"{prefix} underframe", (0.0, 0.0, 0.62), (8.9, 2.0, 0.35), BLACK, 0.08)
    for bogie_x in (-3.25, 3.25):
        box(f"{prefix} bogie {bogie_x}", (bogie_x, 0.0, 0.48), (1.45, 1.75, 0.24), BLACK, 0.07)
        for side in (-1, 1):
            for axle_x in (bogie_x - 0.42, bogie_x + 0.42):
                cylinder(f"{prefix} wheel {bogie_x} {side} {axle_x}", (axle_x, side * 0.93, 0.42), 0.38, 0.16, BLACK)


def roof(prefix):
    box(f"{prefix} roof", (0.0, 0.0, 3.18), (9.15, 2.35, 0.25), SILVER, 0.12)
    box(f"{prefix} roof equipment", (-1.55, 0.0, 3.40), (1.15, 0.82, 0.22), BLACK, 0.06)
    box(f"{prefix} roof fairing", (1.45, 0.0, 3.38), (1.15, 1.2, 0.18), SILVER, 0.05)
    for electrical_x in (-1.55, 1.45):
        box(f"{prefix} electrical base {electrical_x}", (electrical_x, 0.0, 3.57), (0.42, 0.62, 0.10), SILVER, 0.03)
        box(f"{prefix} electrical contact {electrical_x}", (electrical_x, 0.0, 3.70), (0.12, 0.42, 0.08), RED, 0.02)


def rounded_ends(prefix):
    for side in (-1, 1):
        outer = side * 5.0
        inner = side * 4.15
        wedge(
            f"{prefix} rounded red end {side}",
            outer,
            inner,
            2.05,
            2.55,
            0.88,
            3.05,
            RED,
        )


def couplers(prefix):
    box(f"{prefix} front coupler", (5.15, 0.0, 0.72), (0.30, 0.55, 0.30), BLACK, 0.05)
    box(f"{prefix} rear coupler", (-5.15, 0.0, 0.72), (0.30, 0.55, 0.30), BLACK, 0.05)


def make_carriage():
    prefix = "Munich S-Bahn carriage"
    box(prefix + " body", (0.0, 0.0, 1.95), (8.35, 2.55, 2.30), WHITE, 0.15)
    box(prefix + " red waist stripe", (0.0, 0.0, 1.35), (8.55, 2.62, 0.54), RED, 0.04)
    box(prefix + " lower skirt", (0.0, 0.0, 0.88), (8.45, 2.48, 0.30), SILVER, 0.06)
    rounded_ends(prefix)
    side_windows(prefix, (-3.55, -2.35, -1.15, 1.15, 2.35, 3.55))
    doors(prefix, (-3.3, -1.1, 1.1, 3.3))
    underframe(prefix)
    roof(prefix)
    couplers(prefix)
    box(prefix + " rear gangway", (-4.57, 0.0, 1.82), (0.10, 1.25, 1.65), BLACK, 0.04)
    box(prefix + " front gangway", (4.57, 0.0, 1.82), (0.10, 1.25, 1.65), BLACK, 0.04)
    return prefix


def make_front():
    prefix = "Munich S-Bahn front"
    box(prefix + " body", (0.0, 0.0, 1.95), (8.35, 2.55, 2.30), WHITE, 0.15)
    box(prefix + " red waist stripe", (0.0, 0.0, 1.35), (8.55, 2.62, 0.54), RED, 0.04)
    box(prefix + " lower skirt", (0.0, 0.0, 0.88), (8.45, 2.48, 0.30), SILVER, 0.06)
    rounded_ends(prefix)
    for side in (-1, 1):
        box(prefix + " windscreen " + str(side), (side * 4.99, 0.0, 2.36), (0.055, 1.50, 0.82), WINDOW, 0.06)
        box(prefix + " red mask " + str(side), (side * 5.025, 0.0, 1.45), (0.06, 1.85, 0.68), RED_DARK, 0.03)
        for lateral in (-0.62, 0.62):
            box(
                prefix + " headlight " + str(side) + " " + str(lateral),
                (side * 5.07, lateral, 1.28),
                (0.08, 0.20, 0.20),
                LIGHT,
                0.04,
            )
    side_windows(prefix, (-3.55, -2.35, -1.15, 1.15, 2.35, 3.55), z=2.25)
    doors(prefix, (-3.3, -1.1, 1.1, 3.3))
    underframe(prefix)
    roof(prefix)
    couplers(prefix)
    box(prefix + " rear gangway", (-4.57, 0.0, 1.82), (0.10, 1.25, 1.65), BLACK, 0.04)
    box(prefix + " front gangway", (4.57, 0.0, 1.82), (0.10, 1.25, 1.65), BLACK, 0.04)
    return prefix


def export_model(objects, path):
    bpy.ops.object.select_all(action="DESELECT")
    for obj in objects:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = objects[0]
    bpy.ops.export_scene.gltf(
        filepath=path,
        export_format="GLB",
        use_selection=True,
        export_apply=True,
        export_materials="EXPORT",
    )


def build(kind):
    bpy.ops.wm.read_factory_settings(use_empty=True)
    init_materials()
    if kind == "front":
        make_front()
        name = "munich_sbahn_front.glb"
    else:
        make_carriage()
        name = "munich_sbahn_carriage.glb"
    objects = list(bpy.context.scene.objects)
    os.makedirs(OUT, exist_ok=True)
    export_model(objects, os.path.join(OUT, name))


build("front")
build("carriage")
print("Wrote Munich S-Bahn GLBs to", OUT)
