<?php
/** One isolated stand-in for WordPress's process-scoped EDD SDK.
 * activation_limit is fixture input, never a production SQL column.
 */
function focusa_fixture_edd_runtime(PDO $db): void {
    $GLOBALS['focusa_fixture_edd_db'] = $db;
}
function edd_software_licensing(): object {
    return new class {
        public function get_license(int $id): object {
            return new class($GLOBALS['focusa_fixture_edd_db'], $id) {
                public function __construct(private PDO $db, private int $id) {}
                public function get_activation_limit(): mixed {
                    $query = $this->db->prepare('SELECT activation_limit FROM wp_edd_licenses WHERE id=?');
                    $query->execute([$this->id]);
                    return $query->fetchColumn();
                }
            };
        }
    };
}
