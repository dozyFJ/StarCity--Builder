//! Procedural 3D meshes projected into egui: real vertices, perspective and depth-sorted shaded faces.
use egui::{Color32, Painter, Rect, Shape, Stroke, pos2, vec2};
use crate::game::PetKind;
type V = [f32;3];
struct Face { vertices:[V;3], color:[u8;3], edges:[bool;3] }
fn cube(faces:&mut Vec<Face>, c:V, s:V, color:[u8;3]) {
    let v:Vec<V>=(0..8).map(|i| [
        c[0]+if i&1==0 {-s[0]} else {s[0]},
        c[1]+if i&2==0 {-s[1]} else {s[1]},
        c[2]+if i&4==0 {-s[2]} else {s[2]},
    ]).collect();
    for q in [[0,1,3,2],[4,6,7,5],[0,4,5,1],[2,3,7,6],[0,2,6,4],[1,5,7,3]] {
        faces.push(Face{vertices:[v[q[0]],v[q[1]],v[q[2]]],color,edges:[true,true,false]});
        faces.push(Face{vertices:[v[q[0]],v[q[2]],v[q[3]]],color,edges:[false,true,true]});
    }
}
fn egg(faces:&mut Vec<Face>, c:V, size:f32, tint:[u8;3]) {
    let vertex=|i:usize,j:usize| {
        let lat=std::f32::consts::PI*i as f32/12.0;
        let lon=std::f32::consts::TAU*j as f32/20.0;
        let y=lat.cos();
        let radius=lat.sin()*(0.72-0.16*y);
        [c[0]+size*radius*lon.cos(),c[1]+size*y,c[2]+size*radius*lon.sin()]
    };
    for i in 0..12 { for j in 0..20 {
        let color=if (i/3+j/4)%3==0 {[245,225,255]} else {tint};
        let a=vertex(i,j);let b=vertex(i+1,j);let c=vertex(i+1,j+1);let d=vertex(i,j+1);
        faces.push(Face{vertices:[a,b,c],color,edges:[false;3]});faces.push(Face{vertices:[a,c,d],color,edges:[false;3]});
    } }
}
fn draw(p:&Painter,rect:Rect,faces:Vec<Face>,yaw:f32,scale:f32,outline:bool) {
    let transform=|v:V| {
        let x=v[0]*yaw.cos()+v[2]*yaw.sin();
        let z=-v[0]*yaw.sin()+v[2]*yaw.cos();
        [x,v[1]*0.96-z*0.28,v[1]*0.28+z*0.96]
    };
    let center=rect.center()+vec2(0.0,rect.height()*0.04);
    let project=|v:V| { let perspective=4.5/(4.5-v[2]);
        pos2(center.x+v[0]*scale*perspective,center.y-v[1]*scale*perspective) };
    let mut tris:Vec<_>=faces.into_iter().map(|f| {
        let v=f.vertices.map(transform);let depth=(v[0][2]+v[1][2]+v[2][2])/3.0;
        (depth,v,f.color,f.edges)
    }).collect();
    tris.sort_by(|a,b|a.0.total_cmp(&b.0));
    let p=p.with_clip_rect(rect);
    for (_,v,c,edges) in tris {
        let a=[v[1][0]-v[0][0],v[1][1]-v[0][1],v[1][2]-v[0][2]];
        let b=[v[2][0]-v[0][0],v[2][1]-v[0][1],v[2][2]-v[0][2]];
        let normal=[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]];
        let len=(normal.iter().map(|n|n*n).sum::<f32>()).sqrt().max(0.001);
        let light=0.58+0.42*((-normal[0]*0.4+normal[1]*0.7+normal[2]*0.6)/len).abs();
        let color=Color32::from_rgb((c[0] as f32*light) as u8,(c[1] as f32*light) as u8,(c[2] as f32*light) as u8);
        let points=v.map(project);
        let mut mesh=egui::Mesh::default();
        for point in points {mesh.colored_vertex(point,color);}
        mesh.add_triangle(0,1,2);p.add(Shape::mesh(mesh));
        if outline {
            for edge in 0..3 { if edges[edge] {
                p.line_segment([points[edge],points[(edge+1)%3]],Stroke::new(0.8,Color32::from_rgba_unmultiplied(42,49,37,110)));
            } }
        }
    }
}
pub fn pet(p:&Painter,rect:Rect,kind:PetKind,time:f32) {
    let mut faces=Vec::new();let color=kind.color();
    cube(&mut faces,[0.0,-0.2,0.0],[0.62,0.47,0.47],color);
    cube(&mut faces,[0.0,0.37,0.12],[0.58,0.47,0.48],color);
    for x in [-0.37,0.37] {
        cube(&mut faces,[x,-0.7,0.23],[0.2,0.16,0.22],color);
        let ear=if kind==PetKind::Bunny {0.48} else {0.23};
        cube(&mut faces,[x,0.94,0.05],[0.14,ear,0.18],color);
        cube(&mut faces,[x,0.94,0.24],[0.07,ear*0.65,0.02],[255,182,218]);
    }
    for x in [-0.24,0.24] {
        cube(&mut faces,[x,0.4,0.62],[0.09,0.13,0.025],[15,22,47]);
        cube(&mut faces,[x-0.02,0.45,0.655],[0.03,0.04,0.01],[255,255,255]);
    }
    cube(&mut faces,[0.0,0.14,0.64],[0.07,0.045,0.025],[255,142,179]);
    cube(&mut faces,[0.0,-0.13,-0.69],[0.18,0.18,0.35],color);
    if kind==PetKind::Dragon {
        for x in [-0.86,0.86] { cube(&mut faces,[x,0.0,-0.22],[0.33,0.39,0.09],[195,108,255]); }
    }
    if kind==PetKind::Unicorn {
        for i in 0..4 { let s=0.14-i as f32*0.03;
            cube(&mut faces,[0.0,0.88+i as f32*0.12,0.31],[s,0.07,s],[255,225,119]);
        }
    }
    let bob=(time*2.2).sin()*rect.height()*0.025;
    draw(p,rect.translate(vec2(0.0,bob)),faces,0.35+(time*0.7).sin()*0.28,rect.width().min(rect.height())*0.32,false);
}
pub fn egg_preview(p: &Painter, rect: Rect, neon: bool, time: f32, shake: f32) {
    let tint = if neon { [236,166,76] } else { [132,184,94] };
    let mut model = Vec::new();
    egg(&mut model, [0.0,0.0,0.1], 0.88, tint);
    let yaw = 0.3 + (time * 0.5).sin() * 0.2;
    let offset = vec2((time * 43.0).sin() * shake, (time * 31.0).cos() * shake * 0.4);
    draw(p, rect.translate(offset), model, yaw + (time * 35.0).sin() * shake * 0.015,
        rect.width().min(rect.height()) * 0.47, false);
}
