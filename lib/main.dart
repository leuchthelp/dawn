import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:material_ui/material_ui.dart';
import 'package:dawn/src/rust/api/simple.dart';
import 'package:dawn/src/rust/frb_generated.dart';

Future<void> main() async {
  await RustLib.init();
  runApp(ProviderScope(child: const Dawn()));
}

class Dawn extends StatelessWidget {
  const Dawn({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('flutter_rust_bridge quickstart')),
        body: Center(
          child: Text(
            'Action: Call Rust `greet("Tom")`\nResult: `${greet(name: "Tom")}`',
          ),
        ),
      ),
    );
  }
}
