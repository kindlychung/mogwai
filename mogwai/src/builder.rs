use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, Event, HtmlElement, Node, window};
use std::collections::HashMap;

use super::gizmo::Gizmo;
use super::txrx::{Transmitter, Receiver};


#[derive(Clone)]
pub enum Continuous<T> {
  Rx(T, Receiver<T>),
  Static(T)
}


#[derive(Clone)]
pub enum GizmoOption {
  Attribute(String, Continuous<String>),
  Boolean(String, Continuous<bool>),
  Style(String, Continuous<String>),
  Text(Continuous<String>),
  Gizmo(Continuous<GizmoBuilder>)
}


#[derive(Clone)]
pub struct GizmoBuilder {
  tag: String,
  name: String,
  options: Vec<GizmoOption>,
  tx_events: HashMap<String, Transmitter<Event>>,
}


// TODO: Write a macro to generate GizmoBuilder constructors.
// That way we can have one for every tag. #easy-first-issue


pub fn div() -> GizmoBuilder {
  GizmoBuilder::new("div")
}

pub fn h1() -> GizmoBuilder {
  GizmoBuilder::new("h1")
}


pub fn button() -> GizmoBuilder {
  GizmoBuilder::new("button")
}


impl GizmoBuilder {
  pub fn new(tag: &str) -> GizmoBuilder {
    GizmoBuilder {
      name: "unamed_gizmo".into(),
      tag: tag.into(),
      options: vec![],
      tx_events: HashMap::new()
    }
  }

  pub fn named(self, s: &str) -> GizmoBuilder {
    let mut gizmo = self;
    gizmo.name = s.into();
    gizmo
  }

  pub fn option(self, option: GizmoOption) -> GizmoBuilder {
    let mut gizmo = self;
    gizmo.options.push(option);
    gizmo
  }


  pub fn attribute(self, name: &str, value: &str) -> GizmoBuilder {
    self.option(GizmoOption::Attribute(name.to_string(), Continuous::Static(value.to_string())))
  }

  pub fn boolean_attribute(self, name: &str) -> GizmoBuilder {
    self.option(GizmoOption::Boolean(name.to_string(), Continuous::Static(true)))
  }

  pub fn style(self, name: &str, value: &str) -> GizmoBuilder {
    self.option(GizmoOption::Style(name.into(), Continuous::Static(value.into())))
  }

  pub fn text(self, s: &str) -> GizmoBuilder {
    self.option(GizmoOption::Text(Continuous::Static(s.to_string())))
  }

  pub fn with(self, g: GizmoBuilder) -> GizmoBuilder {
    self.option(GizmoOption::Gizmo(Continuous::Static(g)))
  }

  pub fn rx_attribute(self, name: &str, init:&str, value: Receiver<String>) -> GizmoBuilder {
    self.option(GizmoOption::Attribute(name.to_string(), Continuous::Rx(init.into(), value)))
  }

  pub fn rx_boolean_attribute(self, name: &str, init:bool, rx: Receiver<bool>) -> GizmoBuilder {
    self.option(GizmoOption::Boolean(name.to_string(), Continuous::Rx(init, rx)))
  }

  pub fn rx_style(self, name: &str, init:&str, value: Receiver<String>) -> GizmoBuilder {
    self.option(GizmoOption::Style(name.into(), Continuous::Rx(init.into(), value)))
  }

  pub fn rx_text(self, init: &str, s: Receiver<String>) -> GizmoBuilder {
    self.option(GizmoOption::Text(Continuous::Rx(init.into(), s)))
  }

  pub fn rx_with(self, init:GizmoBuilder, g: Receiver<GizmoBuilder>) -> GizmoBuilder {
    self.option(GizmoOption::Gizmo(Continuous::Rx(init, g)))
  }

  pub fn tx_on(self, event: &str, tx: Transmitter<Event>) -> GizmoBuilder {
    let mut b = self;
    b.tx_events.insert(event.into(), tx);
    b
  }

  pub fn build(&self) -> Result<Gizmo, JsValue> {
    trace!("building gizmo");
    let document =
      window().unwrap()
      .document().unwrap();
    let html_el:HtmlElement =
      document
      .create_element(&self.tag)?
      .dyn_into()?;
    let el:&Element =
      html_el
      .dyn_ref()
      .expect("Could not get gizmo element");
    let mut gizmo =
      Gizmo::new(html_el.clone());
    gizmo.name = self.name.clone();
    self
      .tx_events
      .iter()
      .for_each(|(name, tx)| {
        gizmo.tx_on(&name, tx.clone());
      });
    self
      .options
      .iter()
      .fold(
        Ok(()),
        |res, option| {
          res?;
          use Continuous::*;
          use GizmoOption::*;
          match option {
            Attribute(name, Static(value)) => {
              trace!("setting static attribute value on gizmo");
              el.set_attribute(&name, &value)
            }
            Attribute(name, Rx(init, dynamic)) => {
              trace!("setting dynamic attribute value on gizmo");
              gizmo.attribute(&name, &init, dynamic.branch());
              Ok(())
            }
            Boolean(name, Static(should)) => {
              if *should {
                trace!("setting static boolean attribute {:?} on gizmo", name);
                el.set_attribute(&name, "")?;
              }
              Ok(())
            }
            Boolean(name, Rx(init, rx)) => {
              trace!("setting dynamic boolean attribute {:?} on gizmo", name);
              gizmo.boolean_attribute(&name, *init, rx.branch());
              Ok(())
            }
            Style(name, Static(value)) => {
              trace!("setting static style value on gizmo");
              html_el
                .style()
                .set_property(&name, &value)
            }
            Style(name, Rx(init, dynamic)) => {
              trace!("setting dynamic style {} on gizmo", init);
              gizmo.style(&name, &init, dynamic.branch());
              Ok(())
            }
            Text(Static(value)) => {
              trace!("setting static text node on gizmo");
              let text:web_sys::Text =
                web_sys::Text::new_with_data(&value)
                .unwrap();
              html_el
                .dyn_ref::<Node>()
                .unwrap()
                .append_child(text.as_ref())?;
              Ok(())
            }
            Text(Rx(init, dynamic)) => {
              trace!("setting dynamic text node on gizmo");
              gizmo.text(&init, dynamic.branch());
              Ok(())
            }
            Gizmo(Static(static_gizmo_builder)) => {
              let static_gizmo =
                static_gizmo_builder
                .build()?;
              trace!("setting static sub-gizmo on gizmo");
              html_el
                .dyn_ref::<Node>()
                .expect("Could not turn gizmo html_element into Node")
                .append_child(static_gizmo.html_element_ref())?;
              gizmo.static_gizmos.push(static_gizmo);
              Ok(())
            }
            Gizmo(Rx(init_builder, dynamic)) => {
              let init =
                init_builder
                .build()?;
              trace!("setting dynamic sub-gizmo on gizmo");
              gizmo.gizmos(vec![init], dynamic.branch_map(|b| Some(vec![b.clone()])));
              Ok(())
            }
          }
        })?;
    Ok(gizmo)
  }
}