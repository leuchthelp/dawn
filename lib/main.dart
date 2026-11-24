import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:material_ui/material_ui.dart';
import 'package:dawn/src/rust/api/simple.dart';
import 'package:dawn/src/rust/frb_generated.dart';
import 'package:flutter_scene/scene.dart';
import 'package:vector_math/vector_math.dart' as vm;

Future<void> main() async {
  await RustLib.init();
  runApp(ProviderScope(child: const Dawn()));
}

class Dawn extends StatelessWidget {
  const Dawn({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(home: Scaffold(body: FirstScene()));
  }
}

class FirstScene extends StatelessWidget {
  const FirstScene({super.key});

  @override
  Widget build(BuildContext context) {
    return SceneView.declarative(
      loadingBuilder: (context, progress) =>
          const Center(child: CircularProgressIndicator()),
      exposure: 1.4,
      camera: PerspectiveCamera(
        position: vm.Vector3(0, 3.4, 6.2),
        target: vm.Vector3(0, 0.55, 0),
      ),
      children: [
        SceneMesh(
          geometry: SphereGeometry(radius: 0.6),
          material: PhysicallyBasedMaterial()
            ..baseColorFactor =
                vm.Vector4(0.85, 0.30, 0.25, 1.0) // linear RGBA
            ..metallicFactor =
                1.0 // 0 = dielectric, 1 = metal
            ..roughnessFactor = 0.25,
        ),
      ],
    );
  }
}
