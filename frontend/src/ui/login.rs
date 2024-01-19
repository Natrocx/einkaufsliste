use einkaufsliste::model::requests::{LoginUserV1, RegisterUserV1};
use iced::widget::{button, row, text, text_input};
use iced::{Command, Element, Length, Padding};

use super::MainMessage;
use crate::service::api::ApiService;

pub struct LoginView {
  username: String,
  password: String,
  api_service: ApiService,
}

#[derive(Debug, Clone)]
pub enum LoginMessage {
  UsernameChanged(String),
  PasswordChanged(String),
  Login,
  Register,
}

impl LoginView {
  pub fn new(api_service: ApiService) -> Self {
    Self {
      username: String::new(),
      password: String::new(),
      api_service,
    }
  }

  pub fn update(&mut self, message: LoginMessage) -> Command<MainMessage> {
    match message {
      LoginMessage::UsernameChanged(username) => {
        self.username = username;
        Command::none()
      }
      LoginMessage::PasswordChanged(password) => {
        self.password = password;
        Command::none()
      }
      LoginMessage::Login => {
        let api_service = self.api_service.clone();
        let param = LoginUserV1 {
          name: self.username.clone(),
          password: self.password.clone(),
        };

        Command::perform(async move { api_service.login(param).await }, |result| match result {
          Ok(user) => MainMessage::UserChanged(user),
          Err(e) => MainMessage::Toast(e.into()),
        })
      }
      LoginMessage::Register => {
        let api_service = self.api_service.clone();
        let param = RegisterUserV1 {
          name: self.username.clone(),
          password: self.password.clone(),
        };
        Command::perform(
          async move { api_service.register(param).await },
          |result| match result {
            Ok(user) => MainMessage::UserChanged(user),
            Err(e) => MainMessage::Toast(e.into()),
          },
        )
      }
    }
  }

  pub fn view(&self) -> Element<LoginMessage> {
    let username_input = text_input("Username", &self.username)
      .on_input(LoginMessage::UsernameChanged)
      .on_submit(LoginMessage::Login)
      .width(Length::Fill)
      .padding(5)
      .into();

    let password_input = text_input("Password", &self.password)
      .on_input(LoginMessage::PasswordChanged)
      .on_submit(LoginMessage::Login)
      .password()
      .width(Length::Fill)
      .padding(5)
      .into();

    let login_button = button("Login").on_press(LoginMessage::Login).into();
    let register_button = button("Register").on_press(LoginMessage::Register).into();

    iced::widget::column(vec![
      text("Username:").into(),
      username_input,
      text("Password:").into(),
      password_input,
      row(vec![login_button, register_button]).into(),
    ])
    .into()
  }
}
