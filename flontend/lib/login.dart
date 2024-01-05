import 'package:flutter/material.dart';

class LoginView extends StatefulWidget {
  const LoginView({super.key});

  @override
  _LoginViewState createState() => _LoginViewState();
}

class _LoginViewState extends State<LoginView> {
  final TextEditingController _usernameController = TextEditingController();
  final TextEditingController _passwordController = TextEditingController();

  @override
  Widget build(BuildContext context) {
    return Padding(
        padding: const EdgeInsets.all(16),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          const Text("Username:"),
          TextField(
            controller: _usernameController,
          ),
          const Text("Password:"),
          TextField(
            controller: _passwordController,
          ),
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              ElevatedButton(
                onPressed: () {
                  print(
                      "Login: ${_usernameController.text}, ${_passwordController.text}");
                },
                child: Text("Login"),
              ),
              ElevatedButton(
                onPressed: () {
                  print("Register");
                },
                child: Text("Register"),
              ),
            ],
          )
        ]));
  }
}
