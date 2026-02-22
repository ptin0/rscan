import struct
import math
import bpy

BYTES_PER_MES = 4
SCALE = 1e-2
STEP_ANGLE_SIZE = 1.8

data = 0

with open("/home/pitau/data/code/agh/rscanner/client/blender/testfiler.dat", mode="rb") as f:
    data = list(f.read())
    
metadata = data[0:6]
mes = data[6:]
mes = [mes[i:i+BYTES_PER_MES] for i in range(0,len(mes), BYTES_PER_MES)]

point_count = metadata[3]
line_count = metadata[2]
vertices = []

cnt = 0
point = 0
line = 0
side = False
print(len(mes))
for i in mes:
    if not side:
        cnt += 1
        (v,) = struct.unpack(">I", bytes(i))
        vertex = (
            math.sin(math.radians(point * STEP_ANGLE_SIZE)) * v * SCALE * math.cos(math.radians(line * STEP_ANGLE_SIZE)),
            math.cos(math.radians(point * STEP_ANGLE_SIZE)) * v * SCALE * math.cos(math.radians(line * STEP_ANGLE_SIZE)),
            math.sin(math.radians(line * STEP_ANGLE_SIZE)) * v * SCALE
        )
        vertices.append(vertex)
        point += 1
        if point == point_count:
            point -= 1
            side = not side
            line += 1
            continue
    
    if side:
        cnt += 1
        (v,) = struct.unpack(">I", bytes(i))
        vertex = (
            math.sin(math.radians(point * STEP_ANGLE_SIZE)) * v * SCALE * math.cos(math.radians(line * STEP_ANGLE_SIZE)),
            math.cos(math.radians(point * STEP_ANGLE_SIZE)) * v * SCALE * math.cos(math.radians(line * STEP_ANGLE_SIZE)),
            math.sin(math.radians(line * STEP_ANGLE_SIZE)) * v * SCALE
        )
        vertices.append(vertex)
        point -= 1
        if point == -1:
            point += 1
            side = not side
            line += 1
            continue
            
print(cnt)

faces = []
side = False
for line in range(0
, line_count-1):
    for point in range(0, point_count-1):
        if not side:
            face = [
                line * point_count + point,
                line * point_count + point + 1,
                (line + 1) * point_count + point_count - point - 2,
                (line + 1) * point_count + point_count - point - 1,
            ]
            faces.append(face)
    
scan_mesh = bpy.data.meshes.new(name="ScannedMesh")
scan_mesh.from_pydata(vertices, [], faces)
#scan_mesh.from_pydata(vertices, [], []) # for debug
scan_mesh.update()
# make an object
obj = bpy.data.objects.new("new_object", scan_mesh)
# make a collection
new_collection = bpy.data.collections.new('new_collection')
bpy.context.scene.collection.children.link(new_collection)
# add object to scene collection
new_collection.objects.link(obj)

## Optional build modifier addition to show scan order;

buildmod = obj.modifiers.new(name="Build", type="BUILD")
buildmod.frame_start = 1
buildmod.frame_duration = len(mes) + 1
