//! Presets de botões com rótulo, ajuda e semântica de ação.

use super::dialog::{DialogButton, DialogButtonAction};

pub const OK_CANCEL: [DialogButton; 2] = [
    DialogButton::new("OK", "Confirma a ação selecionada", DialogButtonAction::Primary),
    DialogButton::new(
        "Cancelar",
        "Volta ao editor sem confirmar a ação",
        DialogButtonAction::Cancel,
    ),
];

pub const QUIT_UNSAVED: [DialogButton; 3] = [
    DialogButton::new(
        "Salvar",
        "Salva o documento e encerra o editor",
        DialogButtonAction::Primary,
    ),
    DialogButton::new(
        "Não Salvar",
        "Encerra o editor sem salvar as alterações",
        DialogButtonAction::Secondary,
    ),
    DialogButton::new(
        "Cancelar",
        "Volta ao editor sem confirmar a ação",
        DialogButtonAction::Cancel,
    ),
];

pub const DISCARD_NEW: [DialogButton; 2] = [
    DialogButton::new(
        "OK",
        "Descarta alterações e cria um documento em branco",
        DialogButtonAction::Primary,
    ),
    DialogButton::new(
        "Cancelar",
        "Volta ao editor mantendo o documento atual",
        DialogButtonAction::Cancel,
    ),
];

pub const DISCARD_OPEN: [DialogButton; 2] = [
    DialogButton::new(
        "OK",
        "Descarta alterações e abre outro arquivo",
        DialogButtonAction::Primary,
    ),
    DialogButton::new(
        "Cancelar",
        "Volta ao editor mantendo o documento atual",
        DialogButtonAction::Cancel,
    ),
];

pub const DISCARD_CLOSE: [DialogButton; 2] = [
    DialogButton::new(
        "OK",
        "Descarta alterações e fecha o documento",
        DialogButtonAction::Primary,
    ),
    DialogButton::new(
        "Cancelar",
        "Volta ao editor mantendo o documento atual",
        DialogButtonAction::Cancel,
    ),
];

pub const OVERWRITE: [DialogButton; 2] = [
    DialogButton::new(
        "OK",
        "Substitui o arquivo existente no disco",
        DialogButtonAction::Primary,
    ),
    DialogButton::new(
        "Cancelar",
        "Volta ao editor sem confirmar a ação",
        DialogButtonAction::Cancel,
    ),
];

pub const REINTERPRET: [DialogButton; 2] = [
    DialogButton::new(
        "Confirmar",
        "Aplica a codificação escolhida ao documento",
        DialogButtonAction::Primary,
    ),
    DialogButton::new(
        "Cancelar",
        "Volta ao editor sem confirmar a ação",
        DialogButtonAction::Cancel,
    ),
];

pub const CONVERT: [DialogButton; 2] = [
    DialogButton::new(
        "OK",
        "Converte o conteúdo para a codificação escolhida",
        DialogButtonAction::Primary,
    ),
    DialogButton::new(
        "Cancelar",
        "Volta ao editor sem confirmar a ação",
        DialogButtonAction::Cancel,
    ),
];

pub const CONVERT_TABULATION: [DialogButton; 2] = [
    DialogButton::new(
        "Converter",
        "Converte o documento e aplica a opção Para em Tabulação",
        DialogButtonAction::Primary,
    ),
    DialogButton::new(
        "Cancelar",
        "Volta ao editor sem confirmar a ação",
        DialogButtonAction::Cancel,
    ),
];

pub const PATH_OPEN: [DialogButton; 2] = [
    DialogButton::new(
        "OK",
        "Abre o arquivo no caminho digitado",
        DialogButtonAction::Primary,
    ),
    DialogButton::new(
        "Cancelar",
        "Fecha o diálogo sem alterar o documento",
        DialogButtonAction::Cancel,
    ),
];

pub const PATH_SAVE_AS: [DialogButton; 2] = [
    DialogButton::new(
        "OK",
        "Salva o documento no caminho digitado",
        DialogButtonAction::Primary,
    ),
    DialogButton::new(
        "Cancelar",
        "Fecha o diálogo sem alterar o documento",
        DialogButtonAction::Cancel,
    ),
];

pub const FIND: [DialogButton; 2] = [
    DialogButton::new(
        "OK",
        "Busca a próxima ocorrência do texto",
        DialogButtonAction::Primary,
    ),
    DialogButton::new(
        "Cancelar",
        "Fecha o diálogo de busca",
        DialogButtonAction::Cancel,
    ),
];

pub const FIND_REPLACE: [DialogButton; 2] = [
    DialogButton::new(
        "OK",
        "Busca o texto e aplica a substituição",
        DialogButtonAction::Primary,
    ),
    DialogButton::new(
        "Cancelar",
        "Fecha o diálogo de busca",
        DialogButtonAction::Cancel,
    ),
];
