# Refactoring Petit à Petit - Désspaghettification du GUI

## Problèmes Identifiés dans l'Architecture Actuelle

### 1. Explosion des Stores (17 fichiers séparés)
- **Trop de fragmentation** : 17 stores différents pour gérer des données liées
- **Responsabilités floues** : `sceneStore.ts` est une façade qui fait de la délégation
- **Redondance** : Multiple stores qui gèrent des aspects du même domaine
- **Maintenance difficile** : Impossible de suivre le flow des données

### 2. Communication en Couches Multiples
- **Frontend** : `client.ts` → `MainLayout.tsx` → `handleServerMessage()` → delegate vers 5 stores
- **Backend** : `lib.rs` → `ClientManager` → `BuboCoreClient` → TCP
- **Trop de couches** : Message passe par 6-7 couches avant d'arriver au bon endroit

### 3. État Dispersé
- **Scene** : `sceneDataStore` + `sceneStore` + `sceneOperations` + `gridUIStore`
- **Connection** : `connectionStore` + `serverManagerStore` + état local dans `MainLayout`
- **Editor** : `scriptEditorStore` + `compilationStore` + `editorSettingsStore`

## Plan de Refactoring : Simplification Radicale

### Phase 1 : Consolidation des Stores (2 semaines)

**Objectif** : Passer de 17 stores à 4 stores principaux

#### Étape 1.1 : Créer `appStore.ts` - Store Principal
```typescript
// Un seul store pour toute l'application
export const appStore = map<{
  // Connection & Server
  connection: {
    isConnected: boolean;
    serverAddress: string;
    username: string;
    serverState: ServerState;
  };
  
  // Scene & Data
  scene: Scene | null;
  playback: PlaybackState;
  compilation: CompilationState;
  
  // UI State
  ui: {
    currentView: 'editor' | 'grid' | 'split';
    selection: GridSelection;
    panels: PanelState;
    editor: EditorState;
  };
  
  // Peers
  peers: PeersState;
}>({
  // État initial unifié
});
```

#### Étape 1.2 : Migrer les stores un par un
1. **Jour 1-2** : Migrer `connectionStore` + `serverManagerStore` vers `appStore.connection`
2. **Jour 3-4** : Migrer `sceneDataStore` + `playbackStore` vers `appStore.scene` + `appStore.playback`
3. **Jour 5-6** : Migrer `scriptEditorStore` + `compilationStore` vers `appStore.compilation`
4. **Jour 7-8** : Migrer tous les UI stores vers `appStore.ui`
5. **Jour 9-10** : Migrer `peersStore` vers `appStore.peers`

### Phase 2 : Simplification Communication (1 semaine)

#### Étape 2.1 : Éliminer les couches redondantes
**Actuel** : 
```
Frontend → client.ts → MainLayout → handleServerMessage → 5 stores
```

**Cible** : 
```
Frontend → tauri.invoke() → appStore (directement)
```

#### Étape 2.2 : Créer `communication.ts` - Couche Unique
```typescript
// Un seul fichier pour toute la communication
export class BuboCore {
  private store = appStore;
  
  async connect(ip: string, port: number) {
    // Direct tauri invoke
    await invoke('connect_to_server', { ip, port });
    this.store.setKey('connection.isConnected', true);
  }
  
  async sendMessage(message: ClientMessage) {
    await invoke('send_message', { message });
  }
  
  // Message handler unifié
  handleMessage(message: ServerMessage) {
    // Directement dans appStore, plus de délégation
    switch (message.type) {
      case 'Hello':
        this.store.setKey('scene', message.scene);
        this.store.setKey('peers', message.peers);
        break;
      // etc...
    }
  }
}
```

### Phase 3 : Simplification Rust Backend (1 semaine)

#### Étape 3.1 : Éliminer `ClientManager`
**Actuel** : `lib.rs` → `ClientManager` → `BuboCoreClient`
**Cible** : `lib.rs` → `BuboCoreClient` (directement)

#### Étape 3.2 : Fusionner `ServerManager` dans `lib.rs`
- Pas besoin d'un fichier séparé pour 300 lignes
- Intégrer directement dans `lib.rs` avec les autres commandes

### Phase 4 : Restructuration des Composants (1 semaine)

#### Étape 4.1 : Simplifier `MainLayout.tsx`
**Actuel** : 300 lignes avec gestion de state + communication + UI
**Cible** : 100 lignes, juste l'orchestration UI

#### Étape 4.2 : Créer `AppProvider.tsx`
```typescript
// Gestion centralisée de l'état global
export const AppProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [buboCore] = useState(() => new BuboCore());
  
  useEffect(() => {
    // Setup communication
    buboCore.initialize();
  }, []);
  
  return (
    <BuboCoreContext.Provider value={buboCore}>
      {children}
    </BuboCoreContext.Provider>
  );
};
```

## Structure Cible : 4 Fichiers Principaux

### 1. `appStore.ts` (Store Unifié)
- Tout l'état de l'application
- Pas de délégation, pas de façade
- Interface claire et documentée

### 2. `communication.ts` (Communication Unique)
- Une seule classe `BuboCore`
- Interface directe avec Tauri
- Gestion unifiée des messages

### 3. `lib.rs` (Backend Unifié)
- Commands Tauri
- Client TCP direct
- Server management intégré

### 4. `MainLayout.tsx` (UI Orchestration)
- Juste l'interface utilisateur
- Pas de logique métier
- Composants propres

## Avantages de cette Approche

### 1. **Simplicité**
- 4 fichiers principaux au lieu de 20+
- Flow de données linéaire et clair
- Moins de couches d'abstraction

### 2. **Maintenabilité**
- Un seul endroit pour chaque responsabilité
- Pas de délégation complexe
- Debug plus facile

### 3. **Performance**
- Moins de re-renders
- Pas de message passing entre stores
- État centralisé optimisé

### 4. **Respect des Principes Tauri**
- Communication directe Frontend ↔ Backend
- Pas de couches intermédiaires
- Architecture claire et documentée

## Ordre d'Exécution

1. **Semaine 1** : Créer `appStore.ts` et migrer connection/server
2. **Semaine 2** : Migrer scene/playback/compilation vers appStore
3. **Semaine 3** : Créer `communication.ts` et éliminer client.ts
4. **Semaine 4** : Simplifier Rust backend
5. **Semaine 5** : Restructurer MainLayout et composants

## Première Étape Concrète

**Créer `appStore.ts`** avec la structure unifiée et commencer la migration de `connectionStore` + `serverManagerStore`. 

Cette première étape nous permettra de :
- Voir immédiatement les bénéfices
- Valider l'approche
- Continuer progressivement

**Règle d'or** : Chaque étape doit rendre le code plus simple et plus clair, jamais plus complexe.