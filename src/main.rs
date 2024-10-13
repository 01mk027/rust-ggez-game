use crossbeam_utils::thread;
use ggez::audio;
use ggez::audio::AudioContext;
use ggez::audio::SoundSource;
use ggez::context::Has;
use ggez::context::HasMut;
use ggez::event;
use ggez::glam::*;
use ggez::graphics::DrawParam;
use ggez::graphics::Drawable;
use ggez::graphics::PxScale;
use ggez::graphics::Rect;
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};
use lazy_static::lazy_static;
use macroquad::color;
use rand::Rng;
use regex::Regex;
use serialport;
use serialport::new;
use serialport::SerialPort;
use serialport::SerialPortBuilder;
use std::any;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::io::BufRead;
use std::io::BufReader;
use std::ops::Deref;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::time::Instant;
use std::time::{Duration, SystemTime};

type Point2 = Vec2;
type Vector2 = Vec2;

#[derive(Debug)]
struct Bullet {
    x_pos: f32,
    y_pos: f32,
    is_hit_target: bool,
    is_animated: bool,
}

impl Bullet {
    fn new(x_pos: f32) -> Bullet {
        Bullet {
            x_pos,
            y_pos: 4400.0,
            is_hit_target: false,
            is_animated: false,
        }
    }
}

#[derive(Debug, Clone)]
struct First_Target {
    width: f32,
    height: f32,
    hp: f32,
    xst: f32,
    yst: f32,
    is_still_alive: bool,
    is_accurate_hit: bool,
    elapsed_time: Instant,
}

#[derive(Debug)]
struct Target {
    width: f32,
    height: f32,
    mass: f32,
    hp: f32,
    xst: f32,
    yst: f32,
    is_still_alive: bool,
    is_animated: bool,
    animation_start_time: Option<Instant>,
    elapsed_time: Instant,
}

impl Target {
    fn new() -> Target {
        let mut rng = rand::thread_rng();
        let xst: f32 = rng.gen_range(100.0..5750.0);
        Target {
            width: 1700.0,
            height: 1320.0,
            mass: 100.0,
            hp: 40.0,
            xst,
            yst: -1920.0,
            is_still_alive: true,
            is_animated: false,
            animation_start_time: None,
            elapsed_time: Instant::now(),
        }
    }
}

impl First_Target {
    fn new() -> First_Target {
        First_Target {
            width: 1700.0,
            height: 1320.0,
            hp: 360.0,
            xst: 100.0,
            yst: 100.0,
            is_still_alive: true,
            is_accurate_hit: false,
            elapsed_time: Instant::now(),
        }
    }
}

struct Assets {
    fire_sound: audio::Source,
    hit_sound: audio::Source,
}

struct MainState {
    pos_x: f32,
    pos_y: f32,
    att: Box<dyn SerialPort>,
    is_fire_on: u16,
    bullets: Vec<Bullet>,
    assets: Assets,
    earned_points: i32,
    destroyed_enemies: i32
}

impl Assets {
    fn new(ctx: &mut Context) -> GameResult<Assets> {
        let fire_sound = audio::Source::new(ctx, "/boom.ogg").unwrap();
        let hit_sound = audio::Source::new(ctx, "/pew.ogg").unwrap();
        Ok(Assets {
            fire_sound,
            hit_sound,
        })
    }
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        //let sound = audio::Source::new(ctx, "/boom.ogg").unwrap();
        let assets = Assets::new(ctx).unwrap();
        let s = MainState {
            pos_x: 0.0,
            pos_y: 0.0,
            att: serialport::new("/dev/ttyACM0", 57600)
                .data_bits(serialport::DataBits::Eight)
                .timeout(Duration::from_millis(1000))
                .open()
                .expect("Failed to open serial port"),
            is_fire_on: 0,
            bullets: vec![],
            assets,
            earned_points: 0,
            destroyed_enemies: 0
        };

        Ok(s)
    }
}

type T = First_Target;

lazy_static! {
    static ref first_target: Mutex<First_Target> = Mutex::new(First_Target::new());
    static ref targets: Arc<Mutex<Vec<Target>>> = Arc::new(Mutex::new(Vec::new()));
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let mut vx: f32 = 0.0;
        let mut vy: f32 = 0.0;
        let now = SystemTime::now();
        let output = "This is a test.\n".as_bytes();
        self.att.write(output).expect("Write failed!");
        self.att.flush().unwrap();
        let mut reader = BufReader::new(self.att.as_mut());
        let mut my_str = String::new();

        let res = reader.read_line(&mut my_str);
        let re = Regex::new(r"^[0-9]\s+").unwrap();

        match res {
            Ok(v) => {
                //re.replace_all(&my_str, "");
                let v: Vec<&str> = my_str.split_whitespace().collect();

                //fall target(s) onto our hero... literally, this block is added to handle all of target (enemy) operations

                thread::scope(|k| {
                    k.spawn(|_| { 
                        let mut earned_points = &mut self.earned_points;
                        let mut destroyed_enemies = &mut self.destroyed_enemies;
                        //let mut ft = first_target.lock().unwrap();
                        //ft.yst += 9.8;
                        let start = Instant::now();
                        let mut tars = targets.lock().unwrap();

                        match tars.last() {
                            Some(_) => (),
                            None => tars.push(Target::new()),
                        }

                        for tr in tars.iter_mut() {
                            let end = Instant::now();
                            let duration = end.duration_since(tr.elapsed_time);
                            let passed_seconds = duration.as_secs_f64();
                            println!("{:?} {}", tr, passed_seconds);
                            tr.yst += (passed_seconds * passed_seconds * 4.9) as f32;
                            if tr.hp <= 0.0 {
                                *earned_points += 10;
                                *destroyed_enemies += 1;
                                tr.is_still_alive = false;
                                tr.width = 0.0;
                                tr.height = 0.0;
                            }
                            else if tr.hp > 0.0 && tr.hp <= 2.0
                            {
                                tr.is_animated = true;
                                tr.animation_start_time = Some(Instant::now());
                            }
                        }

                        tars.retain(|t| t.yst <= 4400.0 && t.is_still_alive == true);
                        println!("tars length => {}", tars.len());
                        //println!("tars.length => {}", tars.len());
                        //tars.push()
                        // 0.00259291634 = 9.8m

                        //let duration = end.duration_since(start);
                        //let elapsed_seconds =ft.elapsed_time * 100000.0;
                        //
                        //ft.elapsed_time += duration.as_secs_f64();
                        //ft.yst += (elapsed_seconds * elapsed_seconds * 4.9) as f32;
                        //ft.yst += 5.0;
                        //println!("{}", elapsed_seconds)
                        //let duration = end.duration_since(ft.elapsed_time);
                        //ft.yst += 4.9 * duration.as_secs_f32() * duration.as_secs_f32();
                        //println!("{}", duration.as_secs());
                    });
                })
                .unwrap();

                thread::scope(|k| {
                    k.spawn(|_| {
                        let mut bullets = &mut self.bullets;
                        let mut first_target_to_ref = &first_target;
                        let mut ft = first_target.lock().unwrap();
                        let mut tars = targets.lock().unwrap();

                        //TÜM MERMİLERİN YPOSUNU ARTTIR..
                        for i in 0..bullets.len() {
                            bullets[i].y_pos -= 100.0;
                            //firstly try to stop them

                            /*
                            bullets[i].y_pos += 20.0;
                            if bullets[i].y_pos >= screen_height
                            {
                                bullets.remove(i);
                            }
                            else {continue;}
                            */
                            //let is_bullet_exist = bullets.get(i).unwrap();
                            //let f = &first_target.read().unwrap();
                            for tar in tars.iter_mut() {
                                if bullets[i].x_pos >= tar.xst
                                    && bullets[i].x_pos <= (tar.xst + tar.width)
                                    && bullets[i].y_pos <= (tar.yst + tar.height + 120.0)
                                    && bullets[i].y_pos >= tar.yst
                                {
                                    bullets[i].is_animated = true;
                                    if tar.hp <= 0.0 {
                                        bullets[i].is_animated = false;
                                    }

                                    if tar.hp > 0.0 && tar.hp <= 1.0 
                                    {
                                        tar.is_animated = true;
                                        tar.yst -= 0.0;
                                        match tar.animation_start_time{
                                            Some(animation_time) => {
                                                let end = Instant::now();
                                                let duration = end.duration_since(animation_time);
                                                let passed_seconds = duration.as_secs_f64();
                                                if passed_seconds >= 0.85 && passed_seconds <= 1.3{
                                                    tar.is_animated = false;
                                                    tar.is_still_alive = false;
                                                    tar.width = 0.0;
                                                    tar.height = 0.0;
                                                }
                                            },
                                            None => ()
                                        }
                                    }
                                    tar.hp -= 1.0;
                                } else {
                                    bullets[i].is_animated = false;
                                }

                                //println!("{:?}", is_bullet_exist);

                                //mermi koordinatları hedefin içerisindeyse mübadeleye git
                                if bullets[i].x_pos >= tar.xst
                                    && bullets[i].x_pos <= (tar.xst + tar.width)
                                    && bullets[i].y_pos <= (tar.yst + tar.height)
                                    && bullets[i].y_pos >= tar.yst
                                {
                                    bullets[i].is_hit_target = true;
                                } else {
                                    bullets[i].is_hit_target = false;
                                }
                            }
                        }

                        //y-posu ekran boyutundan büyük olanları filtrele (aşmayanları seç)

                        if bullets.len() >= 70 {
                            bullets.retain(|x| x.y_pos >= 0.0 && (x.is_hit_target == false));
                        }

                        bullets.retain(|x| x.is_hit_target == false);
                    });
                })
                .unwrap();

                if v.len() == 3 {
                    let mut is_high: u16 = 0;
                    //let mut kx: u32 = FromStr::from_str(v.get(0).unwrap()).unwrap();
                    //let mut ky: u32 = FromStr::from_str(v.get(1).unwrap()).unwrap();
                    //println!("kx = {}, ky = {}", kx, ky);

                    for (i, ve) in v.iter().enumerate() {
                        //println!("{} -> {}", i, ve.parse::<i32>().unwrap());
                        //println!("{} -> {}", i, ve.parse::<i32>().unwrap());
                        if i == 0 {
                            vx = (5.0 * ve.parse::<i32>().unwrap() as f32) / 1023.0;
                        } else if i == 1 {
                            vy = (5.0 * ve.parse::<i32>().unwrap() as f32) / 1023.0;
                        } else if i == 2 {
                            //self.bullets.push(&Bullet { y_pos: 0.0 });

                            is_high = ve.parse::<u16>().unwrap();

                            if is_high == 1 {
                                let ss = thread::scope(|s| {
                                    s.spawn(|_| {
                                        let mut bullets = &mut self.bullets;

                                        bullets.push(Bullet::new(self.pos_x));
                                        //println!("I am from thread {:?}", now);
                                        //println!("Current length of bullets vector = {}", bullets.len());
                                        //println!("{:?}", bullets);
                                        //&self.sound.play_detached(ctx);
                                        //std::thread::sleep(Duration::from_millis(80));
                                    });
                                })
                                .unwrap();

                                /*
                                //self.bullets.push(&Bullet { y_pos: 0.0 });
                                let handle = thread::spawn(move ||{
                                    // hata burada self.bullets'a erişememek olsa gerek
                                    //self.bullets.push(Bullet::new());


                                    println!("Bullet is added! {:?}", now);
                                });
                                handle.join().unwrap();
                                */
                            }
                        }

                        let venum = ve.parse::<i32>().unwrap();

                        if i == 0 {
                            if venum >= 490 && venum <= 700 {
                                vx = 0.0;
                            } else if venum < 490 {
                                vx = -2.5;
                            } else {
                                vx = 4.5;
                            }
                        }

                        if i == 1 {
                            //println!("{}", venum);
                            if venum >= 300 && venum <= 800 {
                                vy = 0.0;
                            } else if venum < 200 {
                                vy = -140.0;
                            } else {
                                vy = 140.0;
                            }
                        }
                    }
                    /*
                    println!("BİLİ");
                    println!("{} \n {} \n {}", _ctx.gfx.drawable_size().0, _ctx.gfx.drawable_size().1, self.pos_x);
                    println!("BİLİ");
                    */
                    if (self.pos_x <= 0.0 && vy < 0.0) || (self.pos_x >= 5750.0 && vy > 0.0) {
                        vy = 0.0;
                    }
                    let mut tars = targets.lock().unwrap();
                    for tar in tars.iter_mut() {
                        //tar.xst, tar.xst + 1700, tar.yst, tar.yst+650
                        //self.pos_x, self.pos_x + 1700, self.pos_x, self.pos_x

                        if tar.yst + tar.height > 4400.0
                            && tar.xst + 1700.0 >= self.pos_x
                            && tar.xst < self.pos_x + 1750.0 && tar.is_animated == false
                        {
                            ctx.request_quit();
                        } else {
                            println!("You are not fired!");
                        }
                    }

                    self.pos_x = self.pos_x + vy;
                    self.is_fire_on = is_high;
                    //self.pos_y = self.pos_y + vy;
                    //println!("vx = {}, vy = {}, is_high = {} {:?}", vx, vy, is_high, now);

                    my_str.clear();
                }

                /*


                //println!("{:?}", v.len());
                */
            }
            Err(e) => {
                println!("{}", e);
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));
        canvas.set_screen_coordinates(graphics::Rect::new(0.0, 0.0, 7500.0, 4500.0));
        let mut tars = targets.lock().unwrap();
        let earned_points_txt_dest = Vec2::new(200.0, 100.0);
        //let earned_points_txt = graphics::Text::new(format!("Score: {}", self.earned_points));
        let earned_point_text_dest = Point2::new(200.0, 30.0);
        let earned_point_text = format!("Score: {}", self.earned_points);
        let destroyed_enemies_text_dest = Point2::new(4000.0, 30.0);
        let destroyed_enemies_text = format!("Destroyed Enemies: {}", self.destroyed_enemies);

  




        //let coords = [5750.0 / 2.0 - earned_points_txt.width(ctx) as f32 / 2.0, 10.0];
        //let params = graphics::DrawParam::default().dest(coords);
        //let scoreboard_text = graphics::Text::new(format!("L: {} \t R: {}", self.l_score, self.r_score));
         

        let rect = graphics::Rect::new(self.pos_x, 4400.0, 1750.0, 1300.0);
        let mut ft = first_target.lock().unwrap();
        let first_opponent = graphics::Rect::new(ft.xst, ft.yst, ft.width, ft.height);
        for tar in tars.iter_mut() {
            let target_not_animed = graphics::Rect::new(tar.xst, tar.yst, tar.width, tar.height);
            let target_animed = graphics::Rect::new(tar.xst, tar.yst, tar.width, tar.height);
            if !tar.is_animated {
                canvas.draw(
                    &graphics::Quad,
                    graphics::DrawParam::new()
                        .dest(target_not_animed.point())
                        .scale(target_not_animed.size())
                        .color(Color::WHITE),
                );
            }
            else {
                // ...
/* 
                canvas.draw(
                    &graphics::Quad,
                    graphics::DrawParam::new()
                        .dest(target_animed.point())
                        .scale(target_animed.size())
                        .color(Color::RED),
                );
*/

                let circle2 = graphics::Mesh::new_circle(
                    ctx,
                    graphics::DrawMode::fill(),
                    Vec2::new(tar.xst + (tar.width / 2.0), tar.yst + (tar.height / 2.0)),
                    170.0,
                    1.0,
                    Color::from_rgb(255, 90, 0),
                )?;

                canvas.draw(&circle2, graphics::DrawParam::default());

            }
        }

        //let first_target = graphics::Rect::new(self.pos_x, 570.0, 75.0, 30.0);

        let mut color_of_rect = Color::WHITE;
        if self.is_fire_on == 1 {
            //println!("{}", ff.hp);
            color_of_rect = Color::BLACK;
            //self.sound.set_fade_in(Duration::from_millis(1000));
            //self.assets.set_volume(0.1);
            //self.sound.play_detached(ctx).unwrap();
            self.assets.fire_sound.set_volume(0.1);
            self.assets.fire_sound.play_detached(ctx);
        }

        canvas.draw(
            &graphics::Quad,
            graphics::DrawParam::new()
                .dest(rect.point())
                .scale(rect.size())
                .color(color_of_rect),
        );

        let mut color_of_target = Color::WHITE;
        for bullet in &self.bullets {
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                Vec2::new(bullet.x_pos, bullet.y_pos - 19.0),
                50.0,
                2.0,
                Color::RED,
            )?;

            let circle2 = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                Vec2::new(bullet.x_pos, bullet.y_pos),
                300.0,
                2.0,
                Color::from_rgb(255, 90, 0),
            )?;

            if bullet.is_animated && !bullet.is_hit_target && ft.is_still_alive != false {
                canvas.draw(&circle2, graphics::DrawParam::default());
                self.assets.hit_sound.set_volume(0.1);
                self.assets.hit_sound.play_detached(ctx);
            } else {
                canvas.draw(&circle, graphics::DrawParam::default());
            }

        

            

        }


        canvas.draw(
            graphics::Text::new(earned_point_text).set_scale(175.),
            graphics::DrawParam::from(earned_point_text_dest).color(Color::WHITE),
        );

        canvas.draw(
            graphics::Text::new(destroyed_enemies_text).set_scale(175.),
            graphics::DrawParam::from(destroyed_enemies_text_dest).color(Color::WHITE),
        );


       

        //canvas.draw(&graphics::Quad, graphics::DrawParam::new().dest(first_opponent.point()).scale(first_opponent.size()).color(color_of_target));

        //let rectangle: Mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(, color);
        //canvas.draw(&circle, Vec2::new(self.pos_x, self.pos_y));
        //canvas.draw(&rectangle, Into<DrawPa);

        canvas.finish(ctx)?;
        Ok(())
    }
}

pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("super_simple", "ggez");
    let (mut ctx, event_loop) = cb.build()?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state);
}
